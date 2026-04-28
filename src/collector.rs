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
        
        // ── CPU & MEMORY ────────────────────────────────────────────────────
        let cpu_pct = sys.global_cpu_usage();
        let mut per_core: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        let mut count = sys.cpus().len();
        let mut model = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
        let mut freq_mhz: Vec<u64> = sys.cpus().iter().map(|c| c.frequency()).collect();

        // GRACEFUL FALLBACK: If Android blocks sysinfo, scrape the hardware files
        if count == 0 || freq_mhz.iter().all(|&f| f == 0) {
            let (termux_count, termux_freqs) = read_termux_cpu_info();
            if termux_count > 0 {
                count = termux_count;
                freq_mhz = termux_freqs;
                
                // Keep the UI layout intact by filling the blocked percentages with 0
                if per_core.is_empty() {
                    per_core = vec![0.0; count];
                }
                if model.is_empty() {
                    model = "ARM CPU (Hardware Fallback)".into();
                }
            }
        }
        
        let total = sys.total_memory(); let used = sys.used_memory(); let available = sys.available_memory();
        let mem_pct = if total > 0 { used as f32 / total as f32 * 100.0 } else { 0.0 };
        let swap_total = sys.total_swap(); let swap_used = sys.used_swap();
        let swap_pct = if swap_total > 0 { swap_used as f32 / swap_total as f32 * 100.0 } else { 0.0 };
        
        let (stor_total, stor_used, stor_free) = disks.iter().fold((0u64, 0u64, 0u64), |(t, u, f), d| {
            (t + d.total_space(), u + (d.total_space() - d.available_space()), f + d.available_space())
        });
        let stor_pct = if stor_total > 0 { stor_used as f32 / stor_total as f32 * 100.0 } else { 0.0 };
        
        // ── NETWORK IO & SPEED ──────────────────────────────────────────────
        let (mut ts, mut tr): (u64, u64) = networks.iter().fold((0, 0), |(s, r), (_, n)| (s + n.total_transmitted(), r + n.total_received()));
        
        // If sysinfo is blocked by Android and reads 0, try the Termux fallback
        if ts == 0 && tr == 0 {
            if let Some((termux_tx, termux_rx)) = read_termux_net_io() {
                ts = termux_tx;
                tr = termux_rx;
            }
        }

        let now = Instant::now(); let elapsed = now.duration_since(last_t).as_secs_f64();
        let speed_up = if elapsed > 0.0 { (ts.saturating_sub(last_sent)) as f64 / elapsed } else { 0.0 };
        let speed_down = if elapsed > 0.0 { (tr.saturating_sub(last_recv)) as f64 / elapsed } else { 0.0 };
        last_sent = ts; last_recv = tr; last_t = now;
        
        // GRACEFUL FALLBACK: Try Termux API first, then fall back to sysinfo for Linux/Windows
        let ip = read_termux_network().unwrap_or_else(|| {
            networks.iter()
                .flat_map(|(_, n)| n.ip_networks())
                .find(|ip| !ip.addr.is_loopback() && ip.addr.is_ipv4())
                .map(|ip| ip.addr.to_string())
                .unwrap_or_else(|| "N/A".into())
        });

        // ── PROCESSES & DEVICE DATA ─────────────────────────────────────────
        let mut procs: Vec<ProcessInfo> = sys.processes().values().map(|p: &Process| ProcessInfo { pid: p.pid().as_u32(), name: p.name().to_string_lossy().into_owned(), cpu: p.cpu_usage(), mem: p.memory() as f32 / total as f32 * 100.0, status: format!("{:?}", p.status()) }).collect();
        procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal)); procs.truncate(20);
        
        let uptime_secs = System::uptime();
        let battery = read_battery();
        let device = read_device_info();
        
        // ── WRITE TO SHARED STATE ───────────────────────────────────────────
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

// ── TERMUX WRAPPERS ─────────────────────────────────────────────────────────

fn read_termux_cpu_info() -> (usize, Vec<u64>) {
    let mut freqs = Vec::new();
    let mut i = 0;
    loop {
        let dir = format!("/sys/devices/system/cpu/cpu{}", i);
        if !std::path::Path::new(&dir).exists() {
            break;
        }
        
        let cur_freq_path = format!("{}/cpufreq/scaling_cur_freq", dir);
        let max_freq_path = format!("{}/cpufreq/cpuinfo_max_freq", dir);
        
        let mut freq_mhz = 0;
        
        if let Ok(contents) = std::fs::read_to_string(&cur_freq_path) {
            if let Ok(khz) = contents.trim().parse::<u64>() {
                freq_mhz = khz / 1000;
            }
        } else if let Ok(contents) = std::fs::read_to_string(&max_freq_path) {
            if let Ok(khz) = contents.trim().parse::<u64>() {
                freq_mhz = khz / 1000;
            }
        }
        
        freqs.push(freq_mhz);
        i += 1;
    }
    (i, freqs)
}

fn read_termux_net_io() -> Option<(u64, u64)> {
    let mut total_rx = 0;
    let mut total_tx = 0;

    // Method 1: Direct /proc/net/dev parsing
    if let Ok(contents) = std::fs::read_to_string("/proc/net/dev") {
        for line in contents.lines().skip(2) {
            let line_trimmed = line.trim();
            if line_trimmed.starts_with("lo:") || line_trimmed.starts_with("dummy") || line_trimmed.starts_with("tun") { continue; }
            
            let data_str = line.split(':').nth(1).unwrap_or("");
            let data_parts: Vec<&str> = data_str.split_whitespace().collect();
            if data_parts.len() >= 8 {
                total_rx += data_parts[0].parse::<u64>().unwrap_or(0);
                total_tx += data_parts[8].parse::<u64>().unwrap_or(0);
            }
        }
        if total_rx > 0 || total_tx > 0 { return Some((total_tx, total_rx)); }
    }

    // Method 2: 'ip -s link'
    if let Ok(output) = std::process::Command::new("ip").args(["-s", "link"]).output() {
        if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
            let mut lines = stdout.lines();
            while let Some(line) = lines.next() {
                if line.contains("RX: ") {
                    if let Some(next_line) = lines.next() {
                        let parts: Vec<&str> = next_line.split_whitespace().collect();
                        if !parts.is_empty() { total_rx += parts[0].parse::<u64>().unwrap_or(0); }
                    }
                } else if line.contains("TX: ") {
                    if let Some(next_line) = lines.next() {
                        let parts: Vec<&str> = next_line.split_whitespace().collect();
                        if !parts.is_empty() { total_tx += parts[0].parse::<u64>().unwrap_or(0); }
                    }
                }
            }
        }
        if total_rx > 0 || total_tx > 0 { return Some((total_tx, total_rx)); }
    }

    // Method 3: 'ifconfig' parsing
    if let Ok(output) = std::process::Command::new("ifconfig").output() {
        if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                if lower.contains("rx bytes") {
                    let parts: Vec<&str> = lower.split("rx bytes").collect();
                    if parts.len() > 1 {
                        let num_str = parts[1].trim_start_matches(':').trim_start();
                        let end = num_str.find(' ').unwrap_or(num_str.len());
                        total_rx += num_str[..end].parse::<u64>().unwrap_or(0);
                    }
                }
                if lower.contains("tx bytes") {
                    let parts: Vec<&str> = lower.split("tx bytes").collect();
                    if parts.len() > 1 {
                        let num_str = parts[1].trim_start_matches(':').trim_start();
                        let end = num_str.find(' ').unwrap_or(num_str.len());
                        total_tx += num_str[..end].parse::<u64>().unwrap_or(0);
                    }
                }
            }
        }
    }

    if total_rx > 0 || total_tx > 0 {
        Some((total_tx, total_rx))
    } else {
        None
    }
}

fn read_termux_network() -> Option<String> {
    let output = std::process::Command::new("termux-wifi-connectioninfo").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = std::str::from_utf8(&output.stdout).ok()?;
    
    let ip_key = "\"ip\": \"";
    if let Some(start) = stdout.find(ip_key) {
        let rest = &stdout[start + ip_key.len()..];
        if let Some(end) = rest.find('\"') {
            return Some(rest[..end].to_string());
        }
    }
    None
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
    // 1. Try to read Android specific properties first
    let gp = |k: &str| std::process::Command::new("getprop").arg(k).output().ok().and_then(|o| String::from_utf8(o.stdout).ok()).unwrap_or_default().trim().to_string();
    
    let manufacturer = gp("ro.product.manufacturer");
    let model = gp("ro.product.model");
    
    // If we got a manufacturer or model, we are successfully running on Android/Termux
    if !manufacturer.is_empty() || !model.is_empty() {
        let uname = std::process::Command::new("uname").arg("-r").output().ok().and_then(|o| String::from_utf8(o.stdout).ok()).unwrap_or_default().trim().to_string();
        return DeviceInfo { 
            model, 
            android: gp("ro.build.version.release"), 
            arch: gp("ro.product.cpu.abi"), 
            manufacturer, 
            kernel: uname 
        };
    }
    
    // 2. GRACEFUL FALLBACK: We are on Windows, Linux, or macOS
    let os_name = sysinfo::System::name().unwrap_or_else(|| "Unknown".into());
    let os_ver = sysinfo::System::os_version().unwrap_or_default();
    
    DeviceInfo {
        // Map the PC Hostname (e.g., "MOHABBAT-PC") to the manufacturer field
        manufacturer: sysinfo::System::host_name().unwrap_or_else(|| "Localhost".into()),
        model: String::new(),
        // Map the OS Name and Version (e.g., "Windows 11") to the android field
        android: format!("{} {}", os_name, os_ver).trim().to_string(), 
        // Use Rust's built-in target arch (e.g., "x86_64")
        arch: std::env::consts::ARCH.to_string(), 
        kernel: sysinfo::System::kernel_version().unwrap_or_default(),
    }
}


