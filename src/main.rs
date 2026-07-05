use std::sync::{Arc, Mutex, mpsc};

use finalizer::{
    config::data,
    cpu_handle::cpu_stat::CpuStat,
    devices::{self, touch},
    scheduler::manager,
};

fn main() {
    let (tx, rx) = mpsc::channel();
    let log = logger::Logger::new("./log.log");
    let logger_handle = Arc::new(Mutex::new(log));
    let config = data::Config::new("./debug/config.toml");
    let mut id: usize = 0;

    for i in config.clone().policy {
        let tx_clone = tx.clone();
        let log_clone: Arc<Mutex<logger::Logger>> = logger_handle.clone();
        let config_clone = config.clone();
        let _ = std::thread::spawn(move || {
            let mut stat = CpuStat::new(id, i.from, i.to, tx_clone, log_clone, config_clone);
            stat.start_send_event_loop();
        });
        id += 1;
    }

    let devices = devices::touch::find_touchscreen_device().unwrap();

    let touch = touch::Moniter::new(
        devices.as_str(),
        tx.clone(),
        logger_handle.clone(),
        config.clone(),
    );

    let _ = std::thread::spawn(move || {
        touch.start_loop();
    });

    let mut manager = manager::Manager::new(rx, logger_handle, config);
    manager.start_loop();
}
