use core::panic;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, Write},
    sync::{Arc, Mutex},
};

use crate::{
    config::data::{self},
    utils,
};

pub struct CpuFreq {
    pub policys: HashMap<u8, Policy>,
    idle_governor: File,
    logger_handle: Arc<Mutex<logger::Logger>>,
}

impl CpuFreq {
    pub fn new(config: data::Config, logger_handle: Arc<Mutex<logger::Logger>>) -> Self {
        let result = utils::set_file_permissions_numeric(
            "/sys/devices/system/cpu/cpuidle/current_governor",
            0o666,
        );

        match result {
            Ok(_) => {}
            Err(e) => {
                if let Ok(mut log) = logger_handle.lock() {
                    log.error(format!(
                        "无法修改文件权限:/sys/devices/system/cpu/cpuidle/current_governor 错误:{}",
                        e
                    ));
                }
                panic!()
            }
        }

        let result = OpenOptions::new()
            .write(true)
            .open("/sys/devices/system/cpu/cpuidle/current_governor");

        let idle_governor = match result {
            Ok(f) => f,
            Err(e) => {
                if let Ok(mut log) = logger_handle.lock() {
                    log.error(format!(
                        "无法打开文件:/sys/devices/system/cpu/cpuidle/current_governor 错误:{}",
                        e
                    ));
                }
                panic!()
            }
        };

        let mut lines = Vec::new();

        for i in config.policy {
            lines.push(i.from);
        }

        let mut hash_map_policy = HashMap::new();

        for line in lines {
            let policy = Policy::new(line, logger_handle.clone());
            hash_map_policy.insert(line as u8, policy);
        }

        Self {
            policys: hash_map_policy,
            idle_governor,
            logger_handle,
        }
    }

    fn get_policy(&mut self, index: u8) -> &mut Policy {
        let option = self.policys.get_mut(&index);

        match option {
            Some(p) => p,
            None => {
                if let Ok(mut log) = self.logger_handle.lock() {
                    log.error(format!("无法get{}的policy", index));
                }
                panic!()
            }
        }
    }

    pub fn write_index_freq(&mut self, index: u8, values: (u32, u32)) -> Result<(), io::Error> {
        let policy = self.get_policy(index);

        policy.write_min(values.0)?;
        policy.write_max(values.1)?;

        Ok(())
    }

    pub fn write_index_governor(&mut self, index: u8, value: String) -> Result<(), io::Error> {
        let policy = self.get_policy(index);

        policy.write_governor(value)?;

        Ok(())
    }

    pub fn write_idle_governor(&mut self, value: String) -> Result<(), io::Error> {
        self.idle_governor.write_all(value.as_bytes())?;
        self.idle_governor.flush()?;
        Ok(())
    }
}

pub struct Policy {
    max_freq: File,
    min_freq: File,
    governor: File,
}

impl Policy {
    pub fn new(index: u32, logger_handle: Arc<Mutex<logger::Logger>>) -> Self {
        let max_path = format!(
            "/sys/devices/system/cpu/cpufreq/policy{}/scaling_max_freq",
            index
        );
        let min_path = format!(
            "/sys/devices/system/cpu/cpufreq/policy{}/scaling_min_freq",
            index
        );

        let governor_path = format!(
            "/sys/devices/system/cpu/cpufreq/policy{}/scaling_governor",
            index
        );

        let result = utils::set_file_permissions_numeric(max_path.as_str(), 0o666);

        match result {
            Ok(_) => {}
            Err(e) => {
                if let Ok(mut log) = logger_handle.lock() {
                    log.error(format!("无法修改文件权限:{} 错误:{}", max_path, e));
                }
                panic!()
            }
        }

        let result = utils::set_file_permissions_numeric(min_path.as_str(), 0o666);

        match result {
            Ok(_) => {}
            Err(e) => {
                if let Ok(mut log) = logger_handle.lock() {
                    log.error(format!("无法修改文件权限:{} 错误:{}", min_path, e));
                }
                panic!()
            }
        }

        let result = utils::set_file_permissions_numeric(governor_path.as_str(), 0o666);

        match result {
            Ok(_) => {}
            Err(e) => {
                if let Ok(mut log) = logger_handle.lock() {
                    log.error(format!("无法修改文件权限:{} 错误:{}", governor_path, e));
                }
                panic!()
            }
        }

        let result = OpenOptions::new().read(true).write(true).open(&max_path);
        let max_file = match result {
            Ok(f) => f,
            Err(e) => {
                if let Ok(mut log) = logger_handle.lock() {
                    log.error(format!("无法打开文件:{} 错误:{}", max_path, e));
                }
                panic!()
            }
        };

        let result = OpenOptions::new().read(true).write(true).open(&min_path);
        let min_file = match result {
            Ok(f) => f,
            Err(e) => {
                if let Ok(mut log) = logger_handle.lock() {
                    log.error(format!("无法打开文件:{} 错误:{}", min_path, e));
                }
                panic!()
            }
        };

        let result = OpenOptions::new().write(true).open(&governor_path);
        let governor_file = match result {
            Ok(f) => f,
            Err(e) => {
                if let Ok(mut log) = logger_handle.lock() {
                    log.error(format!("无法打开文件:{} 错误:{}", governor_path, e));
                }
                panic!()
            }
        };

        Self {
            max_freq: max_file,
            min_freq: min_file,
            governor: governor_file,
        }
    }

    fn write_max(&mut self, value: u32) -> Result<(), io::Error> {
        self.max_freq.write_all(value.to_string().as_bytes())?;
        self.max_freq.flush()?;
        Ok(())
    }

    fn write_min(&mut self, value: u32) -> Result<(), io::Error> {
        self.min_freq.write_all(value.to_string().as_bytes())?;
        self.min_freq.flush()?;
        Ok(())
    }

    fn write_governor(&mut self, value: String) -> Result<(), io::Error> {
        self.governor.write_all(value.as_bytes())?;
        self.governor.flush()?;
        Ok(())
    }

    pub fn read_max(&mut self) -> Result<u32, io::Error> {
        let mut buffer = String::new();
        self.max_freq.seek(io::SeekFrom::Start(0))?;
        self.max_freq.read_to_string(&mut buffer)?;
        let freq = buffer.trim().parse().unwrap_or(2000000);
        Ok(freq)
    }

    pub fn read_min(&mut self) -> Result<u32, io::Error> {
        let mut buffer = String::new();
        self.max_freq.seek(io::SeekFrom::Start(0))?;
        self.min_freq.read_to_string(&mut buffer)?;
        let freq = buffer.trim().parse().unwrap_or(2000000);
        Ok(freq)
    }
}
