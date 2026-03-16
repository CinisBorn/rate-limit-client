use std::{num::NonZeroU32, ops::Deref};
use crate::{TimeInterval};
use governor::clock::Clock;

pub struct Config {
    pub quota: NonZeroU32,
    pub burst: NonZeroU32,
    pub interval: TimeInterval,
}

impl Copy for Config {}
impl Clone for Config {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct HostConfig {
    pub base: Config,
    pub hostname: &'static str,
}

impl Deref for HostConfig {
    type Target = Config;
    
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Copy for HostConfig {}
impl Clone for HostConfig {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct ConfigWithClock<C: Clock + Clone>{
    pub base: Config,
    pub clock: C,
}

impl<C: Clock + Clone> Deref for ConfigWithClock<C> {
    type Target = Config;
    
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}