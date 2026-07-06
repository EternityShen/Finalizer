use nix::fcntl::{OFlag, open};
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use nix::sys::stat::Mode;
use nix::unistd::read;
use std::mem::size_of;
use std::os::fd::{AsFd, OwnedFd};
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use crate::config::data;
use crate::cpu_handle::cpu_freq::CpuFreq;
use crate::scheduler::manager::Event;

#[repr(C)]
#[derive(Debug)]
pub struct TouchEvent {
    pub tv_sec: i64,
    pub tv_usec: i64,
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

pub fn open_devices(devices: &str) -> OwnedFd {
    open(devices, OFlag::O_RDONLY | OFlag::O_NONBLOCK, Mode::empty()).unwrap()
}

pub struct Moniter {
    devices: OwnedFd,
    logger_handle: Arc<Mutex<logger::Logger>>,
    config: data::Config,
    mode: Arc<AtomicUsize>,
    cpu_freq_handle: CpuFreq,
    tx: mpsc::Sender<Event>,
}

impl Moniter {
    pub fn new(
        devices: &str,
        tx: mpsc::Sender<Event>,
        logger_handle: Arc<Mutex<logger::Logger>>,
        config: data::Config,
        mode: Arc<AtomicUsize>,
    ) -> Self {
        let devices = open_devices(devices);
        let cpu_freq = CpuFreq::new(config.clone(), logger_handle.clone());
        Self {
            devices,
            tx,
            config,
            mode,
            cpu_freq_handle: cpu_freq,
            logger_handle,
        }
    }

    // 修改函数返回值，返回 bool 表示本次是否检测到了触摸事件
    fn touch_monitor(&self) -> bool {
        let event_size = size_of::<TouchEvent>();
        let mut buffer = vec![0u8; event_size];
        let borrowed_fd = self.devices.as_fd();
        let mut poll_fd = [PollFd::new(borrowed_fd, PollFlags::POLLIN)];

        let mut touched = false;

        match poll(&mut poll_fd, PollTimeout::NONE) {
            Ok(n) if n > 0 => {
                if let Some(flags) = poll_fd[0].revents()
                    && flags.contains(PollFlags::POLLIN)
                {
                    // 循环读取，直到把内核缓冲区“榨干”
                    loop {
                        match read(self.devices.as_fd(), &mut buffer) {
                            Ok(bytes_read) if bytes_read == event_size => {
                                // 成功读取到一个事件，标记已收到触摸
                                touched = true;
                                // 注意：这里不要写 break！继续循环读下一个事件
                            }
                            Ok(_) => {
                                // 读取到了不完整的数据或者 EOF
                                break;
                            }
                            Err(nix::Error::EAGAIN) => {
                                // EAGAIN (WouldBlock) 表示当前缓冲区已经没有数据了
                                break;
                            }
                            Err(e) => {
                                if let Ok(mut log) = self.logger_handle.lock() {
                                    log.error(format!("监听屏幕输入事件失败 错误:{}", e));
                                }
                                break; // 发生其他异常也退出，防止死循环
                            }
                        }
                    }
                }
            }
            Ok(_) => {
                println!("未知返回值");
            }
            Err(e) => {
                if let Ok(mut log) = self.logger_handle.lock() {
                    log.error(format!("事件poll失败 错误:{}", e));
                }
            }
        }
        touched
    }

    pub fn start_loop(&mut self) {
        loop {
            // 只有当 touch_monitor 确实排查到了触摸事件，才执行 Boost 逻辑
            if self.touch_monitor() {
                for i in self.config.policy.clone() {
                    let result = self.cpu_freq_handle.policys.get_mut(&(i.from as u8));
                    if let Some(p) = result {
                        let result = p.read_max();
                        match result {
                            Ok(freq) => {
                                if freq < 1200000 {
                                    let result = self
                                        .tx
                                        .send(Event::Boost((i.from as u8, (2000000, 2000000))));

                                    match result {
                                        Ok(_) => {
                                            // println!("Touch");
                                        }
                                        Err(e) => {
                                            if let Ok(mut log) = self.logger_handle.lock() {
                                                log.warn(format!("无法发送Boost事件 错误:{}", e));
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                if let Ok(mut log) = self.logger_handle.lock() {
                                    log.warn(format!("无法读取max_freq 错误:{}", e));
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
            // 限制高频触发，每次消费完一轮事件后休息 200ms
            std::thread::sleep(Duration::from_millis(200));
        }
    }
}

use evdev::enumerate;

pub fn find_touchscreen_device() -> Option<String> {
    for (path, device) in enumerate() {
        let name = device.name().unwrap_or("");
        println!("发现设备: {:?} -> 名字: {}", path, name);

        if let Some(abs_bits) = device.supported_absolute_axes()
            && abs_bits.contains(evdev::AbsoluteAxisCode::ABS_MT_POSITION_X)
        {
            println!("{}", path.to_string_lossy().into_owned());
            return Some(path.to_string_lossy().into_owned());
        }
    }
    None
}
