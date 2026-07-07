use std::fs;

use crate::config::data::{self, GameList};

use super::data::Config;

impl Config {
    pub fn new(path: &str) -> Self {
        let content = fs::read_to_string(path).unwrap();
        toml::from_str(&content).unwrap()
    }

    pub fn get_name(&self) -> data::Name {
        self.name.clone()
    }

    pub fn get_policy(&self) -> Vec<data::Policy> {
        self.policy.clone()
    }

    pub fn get_power(&self) -> data::Power {
        self.mode.power.clone()
    }

    pub fn get_blan(&self) -> data::Blan {
        self.mode.blan.clone()
    }

    pub fn get_perf(&self) -> data::Perf {
        self.mode.perf.clone()
    }

    pub fn get_fast(&self) -> data::Fast {
        self.mode.fast.clone()
    }

    pub fn get_mode(&self) -> data::Mode {
        self.mode.clone()
    }
}

impl GameList {
    pub fn new(path: &str) -> Self {
        let content = fs::read_to_string(path).unwrap();
        toml::from_str(&content).unwrap()
    }
}
