use std::sync::{Arc, Mutex, mpsc};

use crate::{config::data, cpu_handle::cpu_freq::CpuFreq};

pub enum Event {
    Boost((u8, (u32, u32))),
    SetFreq((u8, (u32, u32))),
    SetGovernor((u8, String)),
    SetIdleGovernor(String),
}

pub struct Manager {
    rx: mpsc::Receiver<Event>,
    cpu_freq_handle: CpuFreq,
    logger_handle: Arc<Mutex<logger::Logger>>,
}

impl Manager {
    pub fn new(
        rx: mpsc::Receiver<Event>,
        logger_handle: Arc<Mutex<logger::Logger>>,
        config: data::Config,
    ) -> Self {
        let cpu_freq_handle = CpuFreq::new(config, logger_handle.clone());
        Self {
            rx,
            cpu_freq_handle,
            logger_handle,
        }
    }

    pub fn start_loop(&mut self) {
        loop {
            if let Ok(event) = self.rx.recv() {
                match event {
                    Event::Boost(s) => {
                        // println!("policy{} -> ({} {})", s.0, s.1.0, s.1.1);
                        let result = self.cpu_freq_handle.write_index_freq(s.0, s.1);
                        match result {
                            Ok(_) => {}
                            Err(e) => {
                                if let Ok(mut log) = self.logger_handle.lock() {
                                    log.warn(format!("设置:Policy{} Boost 错误:{}", s.0, e));
                                }
                            }
                        }
                    }
                    Event::SetFreq(s) => {
                        // println!("policy{} -> ({} {})", s.0, s.1.0, s.1.1);
                        let result = self.cpu_freq_handle.write_index_freq(s.0, s.1);
                        match result {
                            Ok(_) => {}
                            Err(e) => {
                                if let Ok(mut log) = self.logger_handle.lock() {
                                    log.warn(format!("设置:Policy{} 频率失败 错误:{}", s.0, e));
                                }
                            }
                        }
                    }
                    Event::SetGovernor(s) => {
                        let result = self.cpu_freq_handle.write_index_governor(s.0, s.1);
                        match result {
                            Ok(_) => {}
                            Err(e) => {
                                if let Ok(mut log) = self.logger_handle.lock() {
                                    log.warn(format!("设置:Policy{} 调速器失败 错误:{}", s.0, e));
                                }
                            }
                        }
                    }
                    Event::SetIdleGovernor(s) => {
                        let result = self.cpu_freq_handle.write_idle_governor(s);
                        match result {
                            Ok(_) => {}
                            Err(e) => {
                                if let Ok(mut log) = self.logger_handle.lock() {
                                    log.warn(format!("设置idle调速器失败 错误{}", e));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
