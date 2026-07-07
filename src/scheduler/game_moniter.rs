use std::{
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::Duration,
};

use crate::{config::data, utils};

pub struct GameMoniter {
    is_game: Arc<AtomicBool>,
    onf: Arc<AtomicBool>,
    game_list: data::GameList,

    logger_handle: Arc<Mutex<logger::Logger>>,
}

impl GameMoniter {
    pub fn new(
        is_game: Arc<AtomicBool>,
        onf: Arc<AtomicBool>,
        game_list: data::GameList,
        logger_handle: Arc<Mutex<logger::Logger>>,
    ) -> Self {
        Self {
            is_game,
            onf,
            game_list,
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

            for line in self.game_list.listvalue.clone() {
                if current_pkg.trim() == line.pkg.trim() {
                    if self.is_game.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    } else {
                        self.is_game
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                        if let Ok(mut log) = self.logger_handle.lock() {
                            log.info(format!(
                                "进入白名单应用: {}:{} 调度关闭",
                                line.name, line.pkg
                            ));
                        }
                    }
                    break;
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
