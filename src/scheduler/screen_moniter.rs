use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize},
        mpsc,
    },
    time::Duration,
};

use crate::{config::data, scheduler::manager, utils};

pub struct Moniter {
    onf: Arc<AtomicBool>,
    logger_handle: Arc<Mutex<logger::Logger>>,
    mode: Arc<AtomicUsize>,
    config: data::Config,
    tx: mpsc::Sender<manager::Event>,
}

impl Moniter {
    pub fn new(
        onf: Arc<AtomicBool>,
        logger_handle: Arc<Mutex<logger::Logger>>,
        mode: Arc<AtomicUsize>,
        config: data::Config,
        tx: mpsc::Sender<manager::Event>,
    ) -> Self {
        Self {
            onf,
            logger_handle,
            mode,
            config,
            tx,
        }
    }

    pub fn start_loop(&mut self) {
        let mut status_temp = false;
        loop {
            let screen_status = utils::monitor_screen_status();
            if screen_status != status_temp {
                if screen_status {
                    if let Ok(mut log) = self.logger_handle.lock() {
                        log.info(format!("屏幕状态: {status_temp} --> {screen_status}"));
                    }
                    self.onf.store(true, std::sync::atomic::Ordering::Relaxed);
                } else {
                    if let Ok(mut log) = self.logger_handle.lock() {
                        log.info(format!("屏幕状态: {status_temp} --> {screen_status}"));
                    }
                    self.onf.store(false, std::sync::atomic::Ordering::Relaxed);

                    match self.mode.load(std::sync::atomic::Ordering::Relaxed) {
                        0 => {
                            for (u, i) in self.config.policy.clone().iter().enumerate() {
                                let config = self.config.mode.power.policy[u].clone();
                                let max_freq = config.sleep_freq;
                                let min_freq = config.min_freq;
                                let result = self.tx.send(manager::Event::SetFreq((
                                    i.from as u8,
                                    (min_freq, max_freq),
                                )));

                                match result {
                                    Ok(_) => {
                                        if let Ok(mut log) = self.logger_handle.lock() {
                                            log.info(format!(
                                                "写入熄屏频率 {}:{}",
                                                min_freq, max_freq
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        if let Ok(mut log) = self.logger_handle.lock() {
                                            log.warn(format!(
                                                "无法写入熄屏频率 {}:{} 错误:{}",
                                                min_freq, max_freq, e
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        1 => {
                            for (u, i) in self.config.policy.clone().iter().enumerate() {
                                let config = self.config.mode.blan.policy[u].clone();
                                let max_freq = config.sleep_freq;
                                let min_freq = config.min_freq;
                                let result = self.tx.send(manager::Event::SetFreq((
                                    i.from as u8,
                                    (min_freq, max_freq),
                                )));

                                match result {
                                    Ok(_) => {
                                        if let Ok(mut log) = self.logger_handle.lock() {
                                            log.info(format!(
                                                "写入熄屏频率 {}:{}",
                                                min_freq, max_freq
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        if let Ok(mut log) = self.logger_handle.lock() {
                                            log.warn(format!(
                                                "无法写入熄屏频率 {}:{} 错误:{}",
                                                min_freq, max_freq, e
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        2 => {
                            for (u, i) in self.config.policy.clone().iter().enumerate() {
                                let config = self.config.mode.perf.policy[u].clone();
                                let max_freq = config.sleep_freq;
                                let min_freq = config.min_freq;
                                let result = self.tx.send(manager::Event::SetFreq((
                                    i.from as u8,
                                    (min_freq, max_freq),
                                )));

                                match result {
                                    Ok(_) => {
                                        if let Ok(mut log) = self.logger_handle.lock() {
                                            log.info(format!(
                                                "写入熄屏频率 {}:{}",
                                                min_freq, max_freq
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        if let Ok(mut log) = self.logger_handle.lock() {
                                            log.warn(format!(
                                                "无法写入熄屏频率 {}:{} 错误:{}",
                                                min_freq, max_freq, e
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        3 => {
                            for (u, i) in self.config.policy.clone().iter().enumerate() {
                                let config = self.config.mode.fast.policy[u].clone();
                                let max_freq = config.sleep_freq;
                                let min_freq = config.min_freq;
                                let result = self.tx.send(manager::Event::SetFreq((
                                    i.from as u8,
                                    (min_freq, max_freq),
                                )));

                                match result {
                                    Ok(_) => {
                                        if let Ok(mut log) = self.logger_handle.lock() {
                                            log.info(format!(
                                                "写入熄屏频率 {}:{}",
                                                min_freq, max_freq
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        if let Ok(mut log) = self.logger_handle.lock() {
                                            log.warn(format!(
                                                "无法写入熄屏频率 {}:{} 错误:{}",
                                                min_freq, max_freq, e
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            if let Ok(mut log) = self.logger_handle.lock() {
                                log.warn("未知模式".to_string());
                            }
                        }
                    }
                }
                status_temp = screen_status;
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    }
}
