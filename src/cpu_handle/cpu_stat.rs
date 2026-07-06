use std::{
    fs::OpenOptions,
    io::Read,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize},
        mpsc::{self},
    },
    time::Duration,
};

use crate::{
    config::data::{self, Mode},
    cpu_handle::cpu_freq::{self},
    scheduler::manager::Event,
};

pub struct CpuStat<'a> {
    policy_id: usize,
    from: u32,
    to: u32,
    history: Vec<(u64, u64)>,
    file_path: &'a str,
    buffer: String,
    tx: mpsc::Sender<Event>,
    logger_handle: Arc<Mutex<logger::Logger>>,
    policy_freq: cpu_freq::Policy,
    mode: Arc<AtomicUsize>,
    onf: Arc<AtomicBool>,
    config: Mode,
}

impl<'a> CpuStat<'a> {
    pub fn new(
        policy_id: usize,
        from: u32,
        to: u32,
        tx: mpsc::Sender<Event>,
        logger_handle: Arc<Mutex<logger::Logger>>,
        config: data::Config,
        mode: Arc<AtomicUsize>,
        onf: Arc<AtomicBool>,
    ) -> Self {
        let num_cpus = if to >= from {
            (to - from + 1) as usize
        } else {
            0
        };

        let freq_handle = cpu_freq::Policy::new(from, logger_handle.clone());

        Self {
            policy_id,
            from,
            to,
            history: vec![(0, 0); num_cpus],
            file_path: "/proc/stat",
            // 复用buffer,预先分配2k内出
            buffer: String::with_capacity(2048),
            tx,
            logger_handle,
            policy_freq: freq_handle,
            mode,
            onf,
            config: config.mode,
        }
    }

    fn get_cpu_load(&mut self) -> u32 {
        self.buffer.clear();

        let mut file = match OpenOptions::new().read(true).open("/proc/stat") {
            Ok(f) => f,
            Err(e) => {
                if let Ok(mut log) = self.logger_handle.lock() {
                    log.error(format!("无法打开文件:/proc/stat 错误:{}", e));
                }
                return 50; // 不 panic，让程序继续运行
            }
        };

        if let Err(e) = file.read_to_string(&mut self.buffer) {
            if let Ok(mut log) = self.logger_handle.lock() {
                log.error(format!("读取:{} 文件错误:{}", self.file_path, e));
            }
            return 50;
        }

        let mut total_load_sum = 0_u64;
        let mut counted_cpus = 0_u32;

        for line in self.buffer.lines() {
            if !line.starts_with("cpu") {
                continue;
            }

            let first_word = line.split_whitespace().next().unwrap_or("");
            if first_word == "cpu" {
                continue;
            }

            //获取cpu编号
            if let Ok(cpu_index) = first_word["cpu".len()..].parse::<u32>()
                && cpu_index >= self.from
                && cpu_index <= self.to
            {
                let history_idx = (cpu_index - self.from) as usize;

                let parts: Vec<u64> = line
                    .split_whitespace()
                    .skip(1)
                    //提升容错：用 filter_map 代替 expect，防止个别行数据破损导致 crash
                    .filter_map(|v| v.parse().ok())
                    .collect();

                if parts.len() < 4 {
                    continue;
                }

                let current_total: u64 = parts.iter().sum();
                let current_idle = parts[3] + parts.get(4).unwrap_or(&0);

                let (prev_total, prev_idle) = self.history[history_idx];

                // 只有当历史有记录，且时间戳推进时才计算（防止高频读取时部分核心未更新导致 total_diff 为 0）
                if prev_total != 0 && current_total > prev_total {
                    let total_diff = current_total - prev_total;
                    let idle_diff = current_idle.saturating_sub(prev_idle);

                    let cpu_load = ((total_diff - idle_diff) * 100) / total_diff;
                    total_load_sum += cpu_load;
                    counted_cpus += 1;
                }
                self.history[history_idx] = (current_total, current_idle);
            }
        }

        if counted_cpus > 0 {
            let avg_load = (total_load_sum / counted_cpus as u64) as u32;
            return avg_load;
        }

        50
    }

    pub fn start_send_event_loop(&mut self) {
        loop {
            // 根据 config_id 匹配对应的策略配置，避免代码复用
            let config_id = self.mode.load(std::sync::atomic::Ordering::Relaxed);

            // 动态提取出当前模式下的具体配置项
            let (delay, min_limit, max_limit, margin, diff) = match config_id {
                0 => {
                    let p = &self.config.power.policy[self.policy_id];

                    (
                        p.delay,
                        p.min_freq as f32,
                        p.max_freq as f32,
                        p.margin,
                        p.diff,
                    )
                }
                1 => {
                    let p = &self.config.blan.policy[self.policy_id];
                    (
                        p.delay,
                        p.min_freq as f32,
                        p.max_freq as f32,
                        p.margin,
                        p.diff,
                    )
                }
                2 => {
                    let p = &self.config.perf.policy[self.policy_id];
                    (
                        p.delay,
                        p.min_freq as f32,
                        p.max_freq as f32,
                        p.margin,
                        p.diff,
                    )
                }
                3 => {
                    let p = &self.config.fast.policy[self.policy_id];
                    (
                        p.delay,
                        p.min_freq as f32,
                        p.max_freq as f32,
                        p.margin,
                        p.diff,
                    )
                }
                _ => {
                    // 未知模式降级
                    (100, 200000.0, 200000.0, 1.0, 700000)
                }
            };

            std::thread::sleep(Duration::from_millis(delay));

            if !self.onf.load(std::sync::atomic::Ordering::Relaxed) {
                continue;
            }

            // 获取当前 CPU 负载
            let load: f32 = (self.get_cpu_load() as f32) / 100.0_f32;
            // println!("load:{}", load);

            // 读取当前系统的硬件最大作为算法基准
            let hardware_max_freq = match self.policy_freq.read_max() {
                Ok(freq) => freq as f32,
                Err(e) => {
                    if let Ok(mut log) = self.logger_handle.lock() {
                        log.error(format!("读取max_freq失败 错误:{}", e));
                    }
                    200000.0 // 降级默认值值
                }
            };

            //核心 DVFS 调频算法
            // 目标频率 = 硬件最大频率 * 当前负载 * 放大系数
            let mut target_freq = hardware_max_freq * load * margin;

            // 将目标频率限制在当前模式配置文件所允许的 [min_limit, max_limit] 区间内
            target_freq = target_freq.clamp(min_limit, max_limit);

            // 设定下发给系统的参数
            // 通常 max 设为算出来的目标频率，min 设为配置下限（允许系统在无负载时自行降频）
            // 如果你的目的是“锁死频率”，则可以将 target_min 也设为 target_freq
            let target_max = target_freq as u32;
            let target_min = min_limit as u32;

            // 计算freq差值
            let diff_freq = target_max.abs_diff(hardware_max_freq as u32);

            // 差值太低,不管它
            if diff_freq < diff {
                continue;
            }

            // println!("{}:{}", target_min, target_max);

            let freq = (target_min, target_max);

            // 发送事件
            let result = self.tx.send(Event::SetFreq((self.from as u8, freq)));

            match result {
                Ok(_) => {}
                Err(e) => {
                    if let Ok(mut log) = self.logger_handle.lock() {
                        log.error(format!("发送频率设置事件失败 错误:{}", e));
                    }
                }
            }
        }
    }
}
