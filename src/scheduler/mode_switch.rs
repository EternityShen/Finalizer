use std::{
    fs::read_to_string,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize},
        mpsc,
    },
};

use crate::{
    config::data,
    scheduler::manager::{self, Event},
    utils,
};

pub struct ModeSwitch {
    mode_path: String,
    mode: Arc<AtomicUsize>,
    tx: mpsc::Sender<manager::Event>,
    config: data::Config,
    is_game: Arc<AtomicBool>,
    logger_handle: Arc<Mutex<logger::Logger>>,
}

impl ModeSwitch {
    pub fn new(
        mode_path: String,
        mode: Arc<AtomicUsize>,
        tx: mpsc::Sender<manager::Event>,
        config: data::Config,
        logger_handle: Arc<Mutex<logger::Logger>>,
        is_game: Arc<AtomicBool>,
    ) -> Self {
        Self {
            mode_path,
            mode,
            tx,
            config,
            is_game,
            logger_handle,
        }
    }

    pub fn start_loop(&mut self) {
        let mut mode_temp = "powersave".to_string();
        loop {
            let mut ino = utils::inotify_init(self.mode_path.clone().as_str());
            utils::inotify_blockage(&mut ino);

            if self.is_game.load(std::sync::atomic::Ordering::Relaxed) {
                continue;
            }

            let result = read_to_string(&self.mode_path);
            let mode = match result {
                Ok(mode_str) => mode_str,
                Err(e) => {
                    if let Ok(mut log) = self.logger_handle.lock() {
                        log.warn(format!("无法获取模式String 错误:{}", e));
                    }
                    "balance".to_string()
                }
            };

            if mode.trim() == mode_temp {
                continue;
            }

            match mode.trim() {
                "powersave" => {
                    self.mode.store(0, std::sync::atomic::Ordering::Relaxed);
                    if let Ok(mut log) = self.logger_handle.lock() {
                        log.info(format!("Mode From:{} --> {}", mode_temp, mode.trim()));
                    }
                    let config = self.config.mode.power.policy.clone();
                    let id_config = self.config.policy.clone();

                    for i in id_config {
                        for l in config.clone() {
                            let result =
                                self.tx.send(Event::SetGovernor((i.from as u8, l.governor)));
                            match result {
                                Ok(_) => {}
                                Err(e) => {
                                    if let Ok(mut log) = self.logger_handle.lock() {
                                        log.warn(format!("无法发送 SetGovernor 事件 错误:{}", e));
                                    }
                                }
                            }
                        }
                    }

                    let result = self.tx.send(Event::SetIdleGovernor(
                        self.config.mode.power.idle_governor.clone(),
                    ));
                    match result {
                        Ok(_) => {}
                        Err(e) => {
                            if let Ok(mut log) = self.logger_handle.lock() {
                                log.warn(format!("无法发送 SetIdleGovernor 事件 错误:{}", e));
                            }
                        }
                    }
                }
                "balance" => {
                    self.mode.store(1, std::sync::atomic::Ordering::Relaxed);
                    if let Ok(mut log) = self.logger_handle.lock() {
                        log.info(format!("Mode From:{} --> {}", mode_temp, mode.trim()));
                    }
                    let config = self.config.mode.blan.policy.clone();
                    let id_config = self.config.policy.clone();

                    for i in id_config {
                        for l in config.clone() {
                            let result =
                                self.tx.send(Event::SetGovernor((i.from as u8, l.governor)));
                            match result {
                                Ok(_) => {}
                                Err(e) => {
                                    if let Ok(mut log) = self.logger_handle.lock() {
                                        log.warn(format!("无法发送 SetGovernor 事件 错误:{}", e));
                                    }
                                }
                            }
                        }
                    }
                    let result = self.tx.send(Event::SetIdleGovernor(
                        self.config.mode.blan.idle_governor.clone(),
                    ));
                    match result {
                        Ok(_) => {}
                        Err(e) => {
                            if let Ok(mut log) = self.logger_handle.lock() {
                                log.warn(format!("无法发送 SetIdleGovernor 事件 错误:{}", e));
                            }
                        }
                    }
                }
                "performance" => {
                    self.mode.store(2, std::sync::atomic::Ordering::Relaxed);
                    if let Ok(mut log) = self.logger_handle.lock() {
                        log.info(format!("Mode From:{} --> {}", mode_temp, mode.trim()));
                    }
                    let config = self.config.mode.perf.policy.clone();
                    let id_config = self.config.policy.clone();

                    for i in id_config {
                        for l in config.clone() {
                            let result =
                                self.tx.send(Event::SetGovernor((i.from as u8, l.governor)));
                            match result {
                                Ok(_) => {}
                                Err(e) => {
                                    if let Ok(mut log) = self.logger_handle.lock() {
                                        log.warn(format!("无法发送 SetGovernor 事件 错误:{}", e));
                                    }
                                }
                            }
                        }
                    }
                    let result = self.tx.send(Event::SetIdleGovernor(
                        self.config.mode.perf.idle_governor.clone(),
                    ));
                    match result {
                        Ok(_) => {}
                        Err(e) => {
                            if let Ok(mut log) = self.logger_handle.lock() {
                                log.warn(format!("无法发送 SetIdleGovernor 事件 错误:{}", e));
                            }
                        }
                    }
                }
                "fast" => {
                    self.mode.store(3, std::sync::atomic::Ordering::Relaxed);
                    if let Ok(mut log) = self.logger_handle.lock() {
                        log.info(format!("Mode From:{} --> {}", mode_temp, mode.trim()));
                    }
                    let config = self.config.mode.perf.policy.clone();
                    let id_config = self.config.policy.clone();

                    for i in id_config {
                        for l in config.clone() {
                            let result =
                                self.tx.send(Event::SetGovernor((i.from as u8, l.governor)));
                            match result {
                                Ok(_) => {}
                                Err(e) => {
                                    if let Ok(mut log) = self.logger_handle.lock() {
                                        log.warn(format!("无法发送 SetGovernor 事件 错误:{}", e));
                                    }
                                }
                            }
                        }
                    }
                    let result = self.tx.send(Event::SetIdleGovernor(
                        self.config.mode.fast.idle_governor.clone(),
                    ));
                    match result {
                        Ok(_) => {}
                        Err(e) => {
                            if let Ok(mut log) = self.logger_handle.lock() {
                                log.warn(format!("无法发送 SetIdleGovernor 事件 错误:{}", e));
                            }
                        }
                    }
                }
                _ => {
                    if let Ok(mut log) = self.logger_handle.lock() {
                        log.warn(format!("UnKnow Mode From:{} --> {}", mode_temp, mode));
                    }
                }
            }

            mode_temp = mode.trim().to_string();
        }
    }
}
