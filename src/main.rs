use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize},
    mpsc,
};

use finalizer::{
    config::data::{self, GameList},
    cpu_handle::cpu_stat::CpuStat,
    devices::{self, touch},
    scheduler::{game_moniter, manager, mode_switch, screen_moniter},
};

fn main() {
    let (tx, rx) = mpsc::channel();
    let mut log = logger::Logger::new("/data/adb/modules/SZE_FINALIZER/log/log.log");
    log.clear();
    log.info("你好!感谢你使用SZE_FINALIZER".to_string());
    let config = data::Config::new("/data/adb/modules/SZE_FINALIZER/config/config.toml");
    log.info(format!("配置名:{}", config.name.name.clone()));
    log.info(format!("配置作者:{}", config.name.author.clone()));
    log.info(format!("配置版本:{}", config.name.version.clone()));

    let logger_handle = Arc::new(Mutex::new(log));

    let mode = Arc::new(AtomicUsize::new(0));
    let onf = Arc::new(AtomicBool::new(true));
    let is_game = Arc::new(AtomicBool::new(false));

    let game_list = GameList::new("/data/adb/modules/SZE_FINALIZER/config/game_list.toml");

    for (id, i) in config.clone().policy.into_iter().enumerate() {
        let tx_clone = tx.clone();
        let log_clone: Arc<Mutex<logger::Logger>> = logger_handle.clone();
        let config_clone = config.clone();
        let mode_clone = mode.clone();
        let onf_clone = onf.clone();
        let is_game_clone = is_game.clone();
        let _ = std::thread::Builder::new()
            .name("listen_1".to_string())
            .spawn(move || {
                let mut stat = CpuStat::new(
                    id,
                    i.from,
                    i.to,
                    tx_clone,
                    log_clone,
                    config_clone,
                    mode_clone,
                    onf_clone,
                    is_game_clone,
                );
                stat.start_send_event_loop();
            });
    }

    let devices = devices::touch::find_touchscreen_device().unwrap();

    let mut touch = touch::Moniter::new(
        devices.as_str(),
        tx.clone(),
        logger_handle.clone(),
        config.clone(),
        mode.clone(),
        is_game.clone(),
    );

    let _ = std::thread::Builder::new()
        .name("touch_listen".to_string())
        .spawn(move || {
            touch.start_loop();
        });

    let mut mode_switch = mode_switch::ModeSwitch::new(
        "/data/adb/modules/SZE_FINALIZER/config/config.txt".to_string(),
        mode.clone(),
        tx.clone(),
        config.clone(),
        logger_handle.clone(),
        is_game.clone(),
    );

    let _ = std::thread::Builder::new()
        .name("mode_switch".to_string())
        .spawn(move || {
            mode_switch.start_loop();
        });

    let mut screen_moniter = screen_moniter::Moniter::new(
        onf.clone(),
        logger_handle.clone(),
        mode.clone(),
        config.clone(),
        tx.clone(),
    );

    let _ = std::thread::Builder::new()
        .name("screen_moniter".to_string())
        .spawn(move || {
            screen_moniter.start_loop();
        });

    let mut game_moniter = game_moniter::GameMoniter::new(
        is_game.clone(),
        onf.clone(),
        game_list,
        logger_handle.clone(),
        tx.clone(),
        config.clone(),
    );

    let _ = std::thread::Builder::new()
        .name("game_moniter".to_string())
        .spawn(move || {
            game_moniter.start_loop();
        });

    let mut manager = manager::Manager::new(rx, logger_handle, config);
    manager.start_loop();
}
