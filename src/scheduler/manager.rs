use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use crate::{
    config::data,
    cpu_handle::cpu_freq::{self, CpuFreq},
};

pub enum Event {
    Boost((u8, (u32, u32))),
    SetFreq((u8, (u32, u32))),
    SetGovernor((u8, String)),
    SetIdleGovernor(String),
}

pub struct Manager {
    rx: mpsc::Receiver<Event>,
    cpu_freq_handle: CpuFreq,
}

impl Manager {
    pub fn new(
        rx: mpsc::Receiver<Event>,
        logger_handle: Arc<Mutex<logger::Logger>>,
        config: data::Config,
    ) -> Self {
        let cpu_freq_handle = CpuFreq::new(config, logger_handle);
        Self {
            rx,
            cpu_freq_handle,
        }
    }

    pub fn start_loop(&mut self) {
        loop {
            if let Ok(event) = self.rx.recv() {
                match event {
                    Event::Boost(s) => {
                        if self
                            .cpu_freq_handle
                            .policys
                            .get_mut(&s.0)
                            .unwrap()
                            .read_max()
                            .unwrap()
                            > 1200000
                        {
                            continue;
                        }
                        self.cpu_freq_handle.write_index_freq(s.0, s.1).unwrap();
                        self.cpu_freq_handle.write_index_freq(4, s.1).unwrap();
                        self.cpu_freq_handle.write_index_freq(7, s.1).unwrap();
                    }
                    Event::SetFreq(s) => {
                        self.cpu_freq_handle.write_index_freq(s.0, s.1).unwrap();
                    }
                    Event::SetGovernor(s) => {}
                    Event::SetIdleGovernor(s) => {}
                }
            }
        }
    }
}
