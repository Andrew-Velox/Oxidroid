use std::sync::{Arc, Mutex};
use crate::{explorer::FileExplorer, settings::Settings, types::SystemData};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab { Overview = 0, Cpu, Memory, Storage, Battery, Network, Processes, Settings }

impl Tab {
    pub const ALL: &'static [Tab] = &[Tab::Overview, Tab::Cpu, Tab::Memory, Tab::Storage, Tab::Battery, Tab::Network, Tab::Processes, Tab::Settings];
    pub fn label(self) -> &'static str {
        match self { Tab::Overview => "Overview", Tab::Cpu => "CPU", Tab::Memory => "Memory", Tab::Storage => "Storage", Tab::Battery => "Battery", Tab::Network => "Network", Tab::Processes => "Processes", Tab::Settings => "Settings" }
    }
}

pub struct App { pub tab: Tab, pub data: Arc<Mutex<SystemData>>, pub explorer: FileExplorer, pub settings: Settings, pub running: bool }

impl App {
    pub fn new() -> Self {
        let start = if std::path::Path::new("/data/data/com.termux/files/home").exists() { "/data/data/com.termux/files/home" } else { "/tmp" };
        Self { tab: Tab::Overview, data: Arc::new(Mutex::new(SystemData::default())), explorer: FileExplorer::new(start), settings: Settings::default(), running: true }
    }
    pub fn idx(&self) -> usize { self.tab as usize }
    pub fn next(&mut self) { self.tab = Tab::ALL[(self.idx() + 1) % Tab::ALL.len()]; }
    pub fn prev(&mut self) { self.tab = Tab::ALL[(self.idx() + Tab::ALL.len() - 1) % Tab::ALL.len()]; }
}