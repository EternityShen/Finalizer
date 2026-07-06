use std::sync::{Arc, Mutex, atomic::AtomicUsize, mpsc};

use finalizer::{
    config::data,
    cpu_handle::cpu_stat::CpuStat,
    devices::{self, touch},
    scheduler::{manager, mode_switch},
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

    for (id, i) in config.clone().policy.into_iter().enumerate() {
        let tx_clone = tx.clone();
        let log_clone: Arc<Mutex<logger::Logger>> = logger_handle.clone();
        let config_clone = config.clone();
        let mode_clone = mode.clone();
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
    );

    let _ = std::thread::Builder::new()
        .name("mode_switch".to_string())
        .spawn(move || {
            mode_switch.start_loop();
        });

    let mut manager = manager::Manager::new(rx, logger_handle, config);
    manager.start_loop();
}
