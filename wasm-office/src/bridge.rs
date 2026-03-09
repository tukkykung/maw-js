//! Data store — JS pushes data in, Macroquad reads it for rendering.
//! For WASM→HTML output, we export state that JS reads and renders as HTML overlays.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct AgentData {
    pub target: String,
    pub name: String,
    pub session: String,
    pub window_index: u32,
    pub active: bool,
    pub status: AgentStatus,
    pub preview: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AgentStatus {
    Busy,
    Ready,
    Idle,
}

impl AgentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentStatus::Busy => "busy",
            AgentStatus::Ready => "ready",
            AgentStatus::Idle => "idle",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "busy" => AgentStatus::Busy,
            "ready" => AgentStatus::Ready,
            _ => AgentStatus::Idle,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RoomData {
    pub name: String,
    pub agent_targets: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct OfficeStats {
    pub total: u32,
    pub busy: u32,
    pub ready: u32,
    pub idle: u32,
}

/// Popup request — WASM tells JS to show/hide popups
#[derive(Clone, Debug)]
pub struct PopupState {
    pub visible: bool,
    pub x: f32,
    pub y: f32,
    pub agent_name: String,
    pub agent_session: String,
    pub agent_status: String,
    pub agent_preview: String,
    pub agent_color: String,
}

impl Default for PopupState {
    fn default() -> Self {
        PopupState {
            visible: false, x: 0.0, y: 0.0,
            agent_name: String::new(), agent_session: String::new(),
            agent_status: String::new(), agent_preview: String::new(),
            agent_color: String::new(),
        }
    }
}

pub struct DataStore {
    pub agents: HashMap<String, AgentData>,
    pub rooms: Vec<RoomData>,
    pub saiyan_targets: Vec<String>,
    pub stats: OfficeStats,
    pub popup: PopupState,
}

impl DataStore {
    pub fn new() -> Self {
        DataStore {
            agents: HashMap::new(),
            rooms: Vec::new(),
            saiyan_targets: Vec::new(),
            stats: OfficeStats::default(),
            popup: PopupState::default(),
        }
    }

    pub fn recompute_stats(&mut self) {
        let (mut busy, mut ready, mut idle) = (0u32, 0u32, 0u32);
        for agent in self.agents.values() {
            match agent.status {
                AgentStatus::Busy => busy += 1,
                AgentStatus::Ready => ready += 1,
                AgentStatus::Idle => idle += 1,
            }
        }
        self.stats = OfficeStats { total: self.agents.len() as u32, busy, ready, idle };
    }

    pub fn recompute_rooms(&mut self) {
        let mut room_map: HashMap<String, Vec<String>> = HashMap::new();
        for agent in self.agents.values() {
            room_map.entry(agent.session.clone()).or_default().push(agent.target.clone());
        }
        self.rooms = room_map.into_iter().map(|(name, targets)| RoomData {
            name, agent_targets: targets,
        }).collect();
        // Sort rooms by name for stable layout
        self.rooms.sort_by(|a, b| a.name.cmp(&b.name));
    }
}
