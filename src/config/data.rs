use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub name: Name,
    pub policy: Vec<Policy>,
    pub mode: Mode,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Name {
    pub name: String,
    pub version: String,
    pub author: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Policy {
    pub from: u32,
    pub to: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Mode {
    pub power: Power,
    pub blan: Blan,
    pub perf: Perf,
    pub fast: Fast,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Power {
    pub idle_governor: String,
    pub policy: Vec<MPolicy>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Blan {
    pub idle_governor: String,
    pub policy: Vec<MPolicy>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Perf {
    pub idle_governor: String,
    pub policy: Vec<MPolicy>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Fast {
    pub idle_governor: String,
    pub policy: Vec<MPolicy>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MPolicy {
    pub delay: u64,
    pub max_freq: u32,
    pub min_freq: u32,
    pub boost_freq: u32,
    pub margin: f32,
    pub diff: u32,
    pub governor: String,
}

#[test]
fn test() {
    let content = std::fs::read_to_string("./debug/config.toml").unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    println!("{:?}", config);
}
