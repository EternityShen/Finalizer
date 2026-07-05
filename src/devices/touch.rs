use libc::sleep;
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
    config_id: AtomicUsize,
    tx: mpsc::Sender<Event>,
}

impl Moniter {
    pub fn new(
        devices: &str,
        tx: mpsc::Sender<Event>,
        logger_handle: Arc<Mutex<logger::Logger>>,
        config: data::Config,
    ) -> Self {
        let devices = open_devices(devices);
        Self {
            devices,
            tx,
            config,
            config_id: AtomicUsize::new(0),
            logger_handle,
        }
    }

    fn touch_monitor(&self) {
        let event_size = size_of::<TouchEvent>();
        let mut buffer = vec![0u8; event_size];
        let borrowed_fd = self.devices.as_fd();
        let mut poll_fd = [PollFd::new(borrowed_fd, PollFlags::POLLIN)];

        match poll(&mut poll_fd, PollTimeout::NONE) {
            Ok(n) if n > 0 => {
                if let Some(flags) = poll_fd[0].revents()
                    && flags.contains(PollFlags::POLLIN)
                {
                    loop {
                        match read(self.devices.as_fd(), &mut buffer) {
                            Ok(bytes_read) if bytes_read == event_size => {
                                break;
                            }
                            Ok(_) => {
                                break;
                            }
                            Err(e) => {
                                if let Ok(mut log) = self.logger_handle.lock() {
                                    log.error(format!("监听屏幕输入事件失败 错误:{}", e));
                                }
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
    }

    pub fn start_loop(&self) {
        loop {
            self.touch_monitor();
            let result = self.tx.send(Event::Boost((0, (2000000, 2000000))));
            std::thread::sleep(Duration::from_millis(200));
            match result {
                Ok(_) => {}
                Err(e) => {
                    if let Ok(mut log) = self.logger_handle.lock() {
                        log.error(format!("发送Touch调度事件失败 错误:{}", e));
                    }
                }
            }
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
