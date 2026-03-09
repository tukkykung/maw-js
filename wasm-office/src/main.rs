mod bridge;

use macroquad::prelude::*;
use bridge::*;
use std::sync::Mutex;

const BG: Color = Color::new(0.04, 0.04, 0.06, 1.0);
const TEXT_DIM: Color = Color::new(1.0, 1.0, 1.0, 0.35);

static DATA: Mutex<Option<DataStore>> = Mutex::new(None);

// ============================================================
// Output buffer — WASM writes here, JS reads via export
// ============================================================
// Simple string buffer for WASM→JS communication
static OUTPUT: Mutex<String> = Mutex::new(String::new());

fn hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
    Color::new(r, g, b, 1.0)
}

fn status_color(status: &AgentStatus) -> Color {
    match status {
        AgentStatus::Busy => hex_color("#fdd835"),
        AgentStatus::Ready => hex_color("#4caf50"),
        AgentStatus::Idle => Color::new(1.0, 1.0, 1.0, 0.3),
    }
}

fn agent_accent(name: &str) -> Color {
    match name {
        "neo" => hex_color("#64b5f6"),
        "nexus" => hex_color("#81c784"),
        "hermes" => hex_color("#ffb74d"),
        "pulse" => hex_color("#4dd0e1"),
        "homelab" => hex_color("#90caf9"),
        "arthur" => hex_color("#ff8a65"),
        "dustboy" => hex_color("#a1887f"),
        "floodboy" => hex_color("#4dd0e1"),
        "fireman" => hex_color("#ef5350"),
        "mother" => hex_color("#ce93d8"),
        "odin" => hex_color("#b39ddb"),
        "volt" | "maeon" => hex_color("#fdd835"),
        "xiaoer" => hex_color("#f48fb1"),
        _ => hex_color("#90a4ae"),
    }
}

fn agent_accent_hex(name: &str) -> &'static str {
    match name {
        "neo" => "#64b5f6",
        "nexus" => "#81c784",
        "hermes" => "#ffb74d",
        "pulse" => "#4dd0e1",
        "homelab" => "#90caf9",
        "arthur" => "#ff8a65",
        "dustboy" => "#a1887f",
        "floodboy" => "#4dd0e1",
        "fireman" => "#ef5350",
        "mother" => "#ce93d8",
        "odin" => "#b39ddb",
        "volt" | "maeon" => "#fdd835",
        "xiaoer" => "#f48fb1",
        _ => "#90a4ae",
    }
}

fn room_color(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "oracles" => hex_color("#64b5f6"),
        "arra" => hex_color("#66bb6a"),
        "hermes" => hex_color("#ffb74d"),
        "brewing" => hex_color("#795548"),
        "watchers" => hex_color("#ce93d8"),
        "tools" => hex_color("#4dd0e1"),
        "solar" => hex_color("#fdd835"),
        _ => hex_color("#78909c"),
    }
}

// ============================================================
// JS → WASM exports: push data in
// ============================================================

/// Push agents. Format: "target|name|session|windowIdx|active|status|preview\n..."
#[no_mangle]
pub extern "C" fn wasm_push_agents(ptr: *const u8, len: usize) {
    let data = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) };
    let mut store = DATA.lock().unwrap();
    let store = store.get_or_insert_with(DataStore::new);
    store.agents.clear();
    for line in data.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 7 { continue; }
        let agent = AgentData {
            target: parts[0].to_string(),
            name: parts[1].to_string(),
            session: parts[2].to_string(),
            window_index: parts[3].parse().unwrap_or(0),
            active: parts[4] == "1",
            status: AgentStatus::from_str(parts[5]),
            preview: parts[6].to_string(),
        };
        store.agents.insert(agent.target.clone(), agent);
    }
    store.recompute_rooms();
    store.recompute_stats();
}

/// Push saiyan targets. Format: "target1\ntarget2\n..."
#[no_mangle]
pub extern "C" fn wasm_push_saiyan(ptr: *const u8, len: usize) {
    let data = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) };
    let mut store = DATA.lock().unwrap();
    let store = store.get_or_insert_with(DataStore::new);
    store.saiyan_targets = data.lines().filter(|l| !l.is_empty()).map(|s| s.to_string()).collect();
}

/// Allocate memory for JS to write strings into
#[no_mangle]
pub extern "C" fn wasm_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn wasm_free(ptr: *mut u8, len: usize) {
    unsafe { drop(Vec::from_raw_parts(ptr, len, len)); }
}

// ============================================================
// WASM → JS exports: JS reads output state
// ============================================================

/// Get popup state as string. Format: "visible|x|y|name|session|status|preview|color"
/// Returns pointer and length via two calls (ptr then len)
#[no_mangle]
pub extern "C" fn wasm_get_popup_ptr() -> *const u8 {
    let out = OUTPUT.lock().unwrap();
    out.as_ptr()
}

#[no_mangle]
pub extern "C" fn wasm_get_popup_len() -> usize {
    OUTPUT.lock().unwrap().len()
}

// ============================================================
// Main render loop
// ============================================================

#[macroquad::main("Oracle Office")]
async fn main() {
    {
        let mut store = DATA.lock().unwrap();
        *store = Some(DataStore::new());
    }

    let mut time = 0.0f32;
    let mut cam_x = 480.0f32;
    let mut cam_y = 450.0f32;
    let mut zoom = 1.0f32;
    let mut hovered_agent: Option<String> = None;
    let mut prev_hovered: Option<String> = None;

    loop {
        let dt = get_frame_time();
        time += dt;
        let sw = screen_width();
        let sh = screen_height();
        let mouse = mouse_position();

        // Zoom
        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            zoom = (zoom + wheel_y * 0.05).clamp(0.3, 3.0);
        }

        // Pan (only when not hovering an agent)
        if is_mouse_button_down(MouseButton::Left) && hovered_agent.is_none() {
            let delta = mouse_delta_position();
            cam_x -= delta.x * sw / zoom * 0.5;
            cam_y -= delta.y * sh / zoom * 0.5;
        }

        clear_background(BG);

        let to_screen = |wx: f32, wy: f32| -> (f32, f32) {
            ((wx - cam_x) * zoom + sw * 0.5, (wy - cam_y) * zoom + sh * 0.5)
        };

        // Lock data
        let store_guard = DATA.lock().unwrap();
        let store = match store_guard.as_ref() {
            Some(s) => s,
            None => { drop(store_guard); next_frame().await; continue; }
        };

        let rooms = &store.rooms;
        let agents_map = &store.agents;
        let saiyan = &store.saiyan_targets;
        let stats = &store.stats;

        // Orbital rings
        let (cx, cy) = to_screen(480.0, 450.0);
        for (r, alpha) in [(150.0, 0.08), (300.0, 0.06), (450.0, 0.04)] {
            draw_circle_lines(cx, cy, r * zoom, 0.5, Color::new(0.5, 0.5, 0.7, alpha));
        }

        // Center
        draw_circle(cx, cy, 7.0 * zoom, hex_color("#26c6da"));
        draw_circle_lines(cx, cy, 45.0 * zoom, 1.0, Color::new(0.15, 0.78, 0.85, 0.15));
        let mc_text = "MISSION CONTROL";
        let mc_w = measure_text(mc_text, None, (12.0 * zoom) as u16, 1.0).width;
        draw_text(mc_text, cx - mc_w / 2.0, cy + 55.0 * zoom, 12.0 * zoom, TEXT_DIM);

        // Rooms + agents
        hovered_agent = None;
        let room_count = rooms.len().max(1);
        for (ri, room) in rooms.iter().enumerate() {
            let angle = (ri as f32 / room_count as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
            let orbit_r = 250.0;
            let (rx, ry) = to_screen(480.0 + angle.cos() * orbit_r, 450.0 + angle.sin() * orbit_r);

            let n = room.agent_targets.len();
            let cluster_r = (50.0 + n as f32 * 12.0) * zoom;
            let rc = room_color(&room.name);

            draw_circle(rx, ry, cluster_r, Color::new(rc.r, rc.g, rc.b, 0.04));
            draw_circle_lines(rx, ry, cluster_r, 1.0, Color::new(rc.r, rc.g, rc.b, 0.15));

            let label = room.name.to_uppercase();
            let lw = measure_text(&label, None, (14.0 * zoom) as u16, 1.0).width;
            draw_text(&label, rx - lw / 2.0, ry - cluster_r - 10.0 * zoom, 14.0 * zoom, rc);

            let ct = format!("{} agent{}", n, if n != 1 { "s" } else { "" });
            let cw = measure_text(&ct, None, (10.0 * zoom) as u16, 1.0).width;
            draw_text(&ct, rx - cw / 2.0, ry + cluster_r + 16.0 * zoom, 10.0 * zoom,
                Color::new(rc.r, rc.g, rc.b, 0.6));

            let agent_r = if n == 1 { 0.0 } else {
                (cluster_r - 35.0 * zoom).min(35.0 * zoom + n as f32 * 6.0 * zoom)
            };
            let avatar_size = 48.0 * zoom;

            for (ai, target) in room.agent_targets.iter().enumerate() {
                let agent = match agents_map.get(target) {
                    Some(a) => a,
                    None => continue,
                };
                let aa = (ai as f32 / n.max(1) as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
                let ax = rx + aa.cos() * agent_r;
                let ay = ry + aa.sin() * agent_r;

                let color = agent_accent(&agent.name);
                let is_saiyan = saiyan.contains(&agent.target);
                let is_busy = agent.status == AgentStatus::Busy;

                // Saiyan glow
                if is_saiyan {
                    let ga = 0.15 + (time * 4.0 + ai as f32).sin().abs() * 0.15;
                    draw_circle(ax, ay, avatar_size * 0.9, Color::new(1.0, 0.85, 0.0, ga));
                }

                // Busy glow
                if is_busy {
                    let ga = 0.06 + (time * 2.0 + ai as f32).sin().abs() * 0.04;
                    draw_circle(ax, ay, avatar_size * 0.7, Color::new(color.r, color.g, color.b, ga));
                }

                // Avatar (colored circle with initial)
                draw_circle(ax, ay, avatar_size * 0.35, color);
                draw_circle_lines(ax, ay, avatar_size * 0.35 + 1.0, 1.5, Color::new(1.0, 1.0, 1.0, 0.3));
                let initial = agent.name.chars().next().unwrap_or('?').to_uppercase().to_string();
                let iw = measure_text(&initial, None, (16.0 * zoom) as u16, 1.0);
                draw_text(&initial, ax - iw.width / 2.0, ay + iw.height / 4.0, 16.0 * zoom, WHITE);

                // Status dot
                draw_circle(ax + avatar_size * 0.3, ay - avatar_size * 0.3, 4.0 * zoom, status_color(&agent.status));

                // Name
                let nw = measure_text(&agent.name, None, (10.0 * zoom) as u16, 1.0).width;
                draw_text(&agent.name, ax - nw / 2.0, ay + avatar_size * 0.45, 10.0 * zoom,
                    if is_busy { color } else { Color::new(1.0, 1.0, 1.0, 0.7) });

                // Hover hit test
                let dx = mouse.0 - ax;
                let dy = mouse.1 - ay;
                if dx * dx + dy * dy < (avatar_size * 0.5) * (avatar_size * 0.5) {
                    hovered_agent = Some(agent.target.clone());
                    draw_circle_lines(ax, ay, avatar_size * 0.45, 2.0 * zoom, color);
                }
            }
        }

        // Write popup state for JS to read (WASM → JS bridge)
        if hovered_agent != prev_hovered {
            let mut out = OUTPUT.lock().unwrap();
            if let Some(ref target) = hovered_agent {
                if let Some(agent) = agents_map.get(target) {
                    *out = format!("1|{:.0}|{:.0}|{}|{}|{}|{}|{}",
                        mouse.0, mouse.1 - 80.0,
                        agent.name, agent.session, agent.status.as_str(),
                        agent.preview, agent_accent_hex(&agent.name));
                }
            } else {
                *out = "0".to_string();
            }
            prev_hovered = hovered_agent.clone();
        }

        // Header
        draw_rectangle(0.0, 0.0, sw, 48.0, Color::new(0.06, 0.06, 0.09, 0.95));
        draw_text("M I S S I O N   C O N T R O L", 24.0, 32.0, 20.0, hex_color("#64b5f6"));

        let live_a = 0.5 + (time * 2.0).sin().abs() * 0.5;
        draw_circle(sw - 300.0, 26.0, 4.0, Color::new(0.2, 0.9, 0.4, live_a));
        draw_text("LIVE", sw - 290.0, 32.0, 13.0, Color::new(0.2, 0.9, 0.4, 1.0));

        let st = format!("{}  agents    {}  rooms", stats.total, rooms.len());
        draw_text(&st, sw - 220.0, 32.0, 13.0, TEXT_DIM);

        // Footer
        draw_rectangle(0.0, sh - 32.0, sw, 32.0, Color::new(0.06, 0.06, 0.09, 0.9));
        draw_circle(120.0, sh - 16.0, 4.0, hex_color("#fdd835"));
        draw_text(&format!("{} busy", stats.busy), 130.0, sh - 10.0, 12.0, TEXT_DIM);
        draw_circle(210.0, sh - 16.0, 4.0, hex_color("#4caf50"));
        draw_text(&format!("{} ready", stats.ready), 220.0, sh - 10.0, 12.0, TEXT_DIM);
        draw_circle(290.0, sh - 16.0, 4.0, Color::new(1.0, 1.0, 1.0, 0.3));
        draw_text(&format!("{} idle", stats.idle), 300.0, sh - 10.0, 12.0, TEXT_DIM);

        if !saiyan.is_empty() {
            draw_circle(380.0, sh - 16.0, 4.0, hex_color("#ff5722"));
            draw_text(&format!("{} saiyan", saiyan.len()), 390.0, sh - 10.0, 12.0, hex_color("#ff5722"));
        }

        let zt = format!("{}%", (zoom * 100.0) as u32);
        let zw = measure_text(&zt, None, 12, 1.0).width;
        draw_text(&zt, sw - zw - 16.0, sh - 10.0, 12.0, TEXT_DIM);
        draw_text(&format!("{}fps", get_fps()), 24.0, sh - 10.0, 12.0, TEXT_DIM);

        // Empty state
        if agents_map.is_empty() {
            let msg = "Waiting for agent data from JS...";
            let mw = measure_text(msg, None, 18, 1.0).width;
            let alpha = 0.3 + (time * 1.5).sin().abs() * 0.3;
            draw_text(msg, sw / 2.0 - mw / 2.0, sh / 2.0, 18.0, Color::new(1.0, 1.0, 1.0, alpha));
        }

        drop(store_guard);
        next_frame().await;
    }
}
