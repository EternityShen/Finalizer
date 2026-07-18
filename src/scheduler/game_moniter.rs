use std::{
    sync::{Arc, Mutex, atomic::AtomicBool, mpsc},
    time::Duration,
};

use crate::{config::data, scheduler::manager, utils};

pub struct GameMoniter {
    is_game: Arc<AtomicBool>,
    onf: Arc<AtomicBool>,
    game_list: data::GameList,
    config: data::Config,
    tx: mpsc::Sender<manager::Event>,
    logger_handle: Arc<Mutex<logger::Logger>>,
}

impl GameMoniter {
    pub fn new(
        is_game: Arc<AtomicBool>,
        onf: Arc<AtomicBool>,
        game_list: data::GameList,
        logger_handle: Arc<Mutex<logger::Logger>>,
        tx: mpsc::Sender<manager::Event>,
        config: data::Config,
    ) -> Self {
        Self {
            is_game,
            onf,
            game_list,
            config,
            tx,
            logger_handle,
        }
    }

    pub fn start_loop(&mut self) {
        loop {
            std::thread::sleep(Duration::from_secs(5));
            if !self.onf.load(std::sync::atomic::Ordering::Relaxed) {
                continue;
            }

            let current_pkg = utils::get_now_top_window_pkg_name();

            'a: for line in self.game_list.listvalue.clone() {
                if current_pkg.contains(line.pkg.trim()) {
                    if self.is_game.load(std::sync::atomic::Ordering::Relaxed) {
                        break 'a;
                    } else {
                        self.is_game
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                        if let Ok(mut log) = self.logger_handle.lock() {
                            log.info(format!(
                                "进入白名单应用: {}:{} 调度关闭",
                                line.name, line.pkg
                            ));
                        }
                        for p in self.config.policy.clone() {
                            let result = self
                                .tx
                                .send(manager::Event::SetFreq((p.from as u8, (500, 8888888))));

                            match result {
                                Ok(_) => {
                                    if let Ok(mut log) = self.logger_handle.lock() {
                                        log.info(format!("Policy: {} 频率恢复成功", p.from));
                                    }
                                }
                                Err(e) => {
                                    if let Ok(mut log) = self.logger_handle.lock() {
                                        log.warn(format!(
                                            "Policy: {} 频率恢复失败 错误: {}",
                                            p.from, e
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                if let Ok(mut log) = self.logger_handle.lock()
                    && self.is_game.load(std::sync::atomic::Ordering::Relaxed)
                {
                    self.is_game
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                    log.info("退出白名单应用调度开启".to_string());
                }
            }
        }
    }
}
