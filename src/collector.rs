use std::{sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
use sysinfo::{Disks, Networks, Process, System};
use crate::types::*;

pub fn collect_loop(data: Arc<Mutex<SystemData>>) {
    let mut sys = System::new_all();
    let mut disks = Disks::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();
    let mut last_sent = 0u64; let mut last_recv = 0u64; let mut last_t = Instant::now();
    
    loop {
        sys.refresh_all(); disks.refresh_list(); networks.refresh_list();
        
        let cpu_pct = sys.global_cpu_usage();
        let per_core: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        let count = sys.cpus().len();
        let model = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
        let freq_mhz: Vec<u64> = sys.cpus().iter().map(|c| c.frequency()).collect();
        
        let total = sys.total_memory(); let used = sys.used_memory(); let available = sys.available_memory();
        let mem_pct = if total > 0 { used as f32 / total as f32 * 100.0 } else { 0.0 };
        let swap_total = sys.total_swap(); let swap_used = sys.used_swap();
        let swap_pct = if swap_total > 0 { swap_used as f32 / swap_total as f32 * 100.0 } else { 0.0 };
        
        let (stor_total, stor_used, stor_free) = disks.iter().fold((0u64, 0u64, 0u64), |(t, u, f), d| {
            (t + d.total_space(), u + (d.total_space() - d.available_space()), f + d.available_space())
        });
        let stor_pct = if stor_total > 0 { stor_used as f32 / stor_total as f32 * 100.0 } else { 0.0 };
        
        let (ts, tr): (u64, u64) = networks.iter().fold((0, 0), |(s, r), (_, n)| (s + n.total_transmitted(), r + n.total_received()));
        let now = Instant::now(); let elapsed = now.duration_since(last_t).as_secs_f64();
        let speed_up = if elapsed > 0.0 { (ts.saturating_sub(last_sent)) as f64 / elapsed } else { 0.0 };
        let speed_down = if elapsed > 0.0 { (tr.saturating_sub(last_recv)) as f64 / elapsed } else { 0.0 };
        last_sent = ts; last_recv = tr; last_t = now;
        
        // ── IP Fallback Logic ──────────────────────────────────────────────
        let ip = read_termux_network().unwrap_or_else(|| {
            networks.iter()
                .flat_map(|(_, n)| n.ip_networks())
                .find(|ip| !ip.addr.is_loopback() && ip.addr.is_ipv4())
                .map(|ip| ip.addr.to_string())
                .unwrap_or_else(|| "N/A".into())
        });
        
        let mut procs: Vec<ProcessInfo> = sys.processes().values().map(|p: &Process| ProcessInfo { pid: p.pid().as_u32(), name: p.name().to_string_lossy().into_owned(), cpu: p.cpu_usage(), mem: p.memory() as f32 / total as f32 * 100.0, status: format!("{:?}", p.status()) }).collect();
        procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal)); procs.truncate(20);
        
        let uptime_secs = System::uptime();
        let battery = read_battery();
        let device = read_device_info();
        
        let mut d = data.lock().unwrap();
        d.cpu = CpuData { percent: cpu_pct, per_core, count, model, freq_mhz };
        d.memory = MemData { total, used, available, percent: mem_pct, swap_total, swap_used, swap_percent: swap_pct };
        d.storage = StorageData { total: stor_total, used: stor_used, free: stor_free, percent: stor_pct };
        d.network = NetData { ip, bytes_sent: ts, bytes_recv: tr, speed_up, speed_down };
        d.processes = procs; d.battery = battery; d.uptime_secs = uptime_secs;
        if d.device.kernel.is_empty() { d.device = device; }
        drop(d);
        
        thread::sleep(Duration::from_millis(500));
    }
}

fn read_battery() -> BatteryData {
    if let Ok(out) = std::process::Command::new("termux-battery-status").output() {
        if let Ok(s) = std::str::from_utf8(&out.stdout) {
            fn extract<'a>(s: &'a str, key: &str) -> Option<&'a str> {
                let k = format!("\"{}\"", key); let pos = s.find(&k)?;
                let rest = &s[pos + k.len()..]; let colon = rest.find(':')? + 1;
                let val = rest[colon..].trim();
                if val.starts_with('"') { let end = val[1..].find('"')?; Some(&val[1..=end]) } else { let end = val.find(|c: char| c == ',' || c == '}').unwrap_or(val.len()); Some(val[..end].trim()) }
            }
            return BatteryData {
                percentage: extract(s, "percentage").and_then(|v| v.parse().ok()).unwrap_or(0),
                status: extract(s, "status").unwrap_or("Unknown").to_string(),
                health: extract(s, "health").unwrap_or("Unknown").to_string(),
                temperature: extract(s, "temperature").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                plugged: extract(s, "plugged").unwrap_or("Unknown").to_string(),
                current_ua: extract(s, "current").and_then(|v| v.parse().ok()).unwrap_or(0),
                time_remaining: "N/A".into(),
            };
        }
    }
    BatteryData { percentage: 0, status: "N/A".into(), health: "N/A".into(), temperature: 0.0, plugged: "N/A".into(), current_ua: 0, time_remaining: "N/A".into() }
}

fn read_device_info() -> DeviceInfo {
    let gp = |k: &str| std::process::Command::new("getprop").arg(k).output().ok().and_then(|o| String::from_utf8(o.stdout).ok()).unwrap_or_default().trim().to_string();
    let uname = std::process::Command::new("uname").arg("-r").output().ok().and_then(|o| String::from_utf8(o.stdout).ok()).unwrap_or_default().trim().to_string();
    DeviceInfo { model: gp("ro.product.model"), android: gp("ro.build.version.release"), arch: gp("ro.product.cpu.abi"), manufacturer: gp("ro.product.manufacturer"), kernel: uname }
}

// ── New Helper Function for Termux Wi-Fi ─────────────────────────────
fn read_termux_network() -> Option<String> {
    // Attempt to run the Termux API command. Fails silently if not installed/not on Termux.
    let output = std::process::Command::new("termux-wifi-connectioninfo")
        .output()
        .ok()?; 

    if !output.status.success() {
        return None;
    }

    let stdout = std::str::from_utf8(&output.stdout).ok()?;
    
    // Quick extraction of the "ip" field from the JSON output
    let ip_key = "\"ip\": \"";
    if let Some(start) = stdout.find(ip_key) {
        let rest = &stdout[start + ip_key.len()..];
        if let Some(end) = rest.find('\"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}