use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, List, ListItem, Paragraph, Row, Table,
    },
    Frame, Terminal,
};
use std::{
    fs,
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use sysinfo::{Disks, Networks, Process, System};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab { Overview=0, Cpu, Memory, Storage, Battery, Network, Processes, Settings }

impl Tab {
    const ALL: &'static [Tab] = &[Tab::Overview,Tab::Cpu,Tab::Memory,Tab::Storage,Tab::Battery,Tab::Network,Tab::Processes,Tab::Settings];
    fn label(self) -> &'static str {
        match self { Tab::Overview=>"Overview",Tab::Cpu=>"CPU",Tab::Memory=>"Memory",Tab::Storage=>"Storage",Tab::Battery=>"Battery",Tab::Network=>"Network",Tab::Processes=>"Processes",Tab::Settings=>"Settings" }
    }
}

#[derive(Default, Clone)]
struct CpuData { percent:f32, per_core:Vec<f32>, count:usize, model:String, freq_mhz:Vec<u64> }
#[derive(Default, Clone)]
struct MemData { total:u64, used:u64, available:u64, percent:f32, swap_total:u64, swap_used:u64, swap_percent:f32 }
#[derive(Default, Clone)]
struct StorageData { total:u64, used:u64, free:u64, percent:f32 }
#[derive(Default, Clone)]
struct BatteryData { percentage:u8, status:String, health:String, temperature:f32, plugged:String, current_ua:i64, time_remaining:String }
#[derive(Default, Clone)]
struct NetData { ip:String, bytes_sent:u64, bytes_recv:u64, speed_up:f64, speed_down:f64 }
#[derive(Default, Clone)]
struct ProcessInfo { pid:u32, name:String, cpu:f32, mem:f32, status:String }
#[derive(Default, Clone)]
struct DeviceInfo { model:String, android:String, arch:String, manufacturer:String, kernel:String }
#[derive(Default, Clone)]
struct SystemData { cpu:CpuData, memory:MemData, storage:StorageData, battery:BatteryData, network:NetData, processes:Vec<ProcessInfo>, device:DeviceInfo, uptime_secs:u64 }

#[derive(Clone)]
struct FileEntry { name:String, path:PathBuf, is_dir:bool, size:u64, count:usize }

struct FileExplorer { current_path:PathBuf, items:Vec<FileEntry>, selected:usize, offset:usize, focused:bool }
impl FileExplorer {
    fn new(start:&str) -> Self {
        let mut fe = Self { current_path:PathBuf::from(start), items:Vec::new(), selected:0, offset:0, focused:false };
        fe.refresh(); fe
    }
    fn refresh(&mut self) {
        self.items.clear();
        self.items.push(FileEntry { name:"..".into(), path:self.current_path.parent().unwrap_or(&self.current_path).to_path_buf(), is_dir:true, size:0, count:0 });
        if let Ok(rd) = fs::read_dir(&self.current_path) {
            let mut entries:Vec<FileEntry> = rd.flatten().filter_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_string_lossy().into_owned();
                let is_dir = path.is_dir();
                let (size,count) = if is_dir { (0, fs::read_dir(&path).map(|d|d.count()).unwrap_or(0)) } else { (fs::metadata(&path).map(|m|m.len()).unwrap_or(0), 0) };
                Some(FileEntry { name, path, is_dir, size, count })
            }).collect();
            entries.sort_by(|a,b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
            self.items.extend(entries);
        }
    }
    fn up(&mut self) { if self.selected>0 { self.selected-=1; } }
    fn down(&mut self) { if self.selected+1<self.items.len() { self.selected+=1; } }
    fn enter(&mut self) { if let Some(e)=self.items.get(self.selected) { if e.is_dir { self.current_path=e.path.clone(); self.selected=0; self.offset=0; self.refresh(); } } }
    fn visible<'a>(&'a mut self, max:usize) -> &'a [FileEntry] {
        if self.selected<self.offset { self.offset=self.selected; }
        else if self.selected>=self.offset+max { self.offset=self.selected+1-max; }
        let end=(self.offset+max).min(self.items.len());
        &self.items[self.offset..end]
    }
}

struct Settings { refresh_ms:u64, battery_mah:u32, selected:usize }
impl Default for Settings { fn default()->Self { Self { refresh_ms:500, battery_mah:4000, selected:0 } } }

struct App { tab:Tab, data:Arc<Mutex<SystemData>>, explorer:FileExplorer, settings:Settings, running:bool }
impl App {
    fn new()->Self {
        let start = if std::path::Path::new("/data/data/com.termux/files/home").exists() { "/data/data/com.termux/files/home" } else { "/tmp" };
        Self { tab:Tab::Overview, data:Arc::new(Mutex::new(SystemData::default())), explorer:FileExplorer::new(start), settings:Settings::default(), running:true }
    }
    fn idx(&self)->usize { self.tab as usize }
    fn next(&mut self) { self.tab=Tab::ALL[(self.idx()+1)%Tab::ALL.len()]; }
    fn prev(&mut self) { self.tab=Tab::ALL[(self.idx()+Tab::ALL.len()-1)%Tab::ALL.len()]; }
}

fn collect_loop(data:Arc<Mutex<SystemData>>) {
    let mut sys = System::new_all();
    let mut disks = Disks::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();
    let mut last_sent=0u64; let mut last_recv=0u64; let mut last_t=Instant::now();
    loop {
        sys.refresh_all(); disks.refresh_list(); networks.refresh_list();
        let cpu_pct=sys.global_cpu_usage();
        let per_core:Vec<f32>=sys.cpus().iter().map(|c|c.cpu_usage()).collect();
        let count=sys.cpus().len();
        let model=sys.cpus().first().map(|c|c.brand().to_string()).unwrap_or_default();
        let freq_mhz:Vec<u64>=sys.cpus().iter().map(|c|c.frequency()).collect();
        let total=sys.total_memory(); let used=sys.used_memory(); let available=sys.available_memory();
        let mem_pct=if total>0{used as f32/total as f32*100.0}else{0.0};
        let swap_total=sys.total_swap(); let swap_used=sys.used_swap();
        let swap_pct=if swap_total>0{swap_used as f32/swap_total as f32*100.0}else{0.0};
        let (stor_total,stor_used,stor_free)=disks.iter().fold((0u64,0u64,0u64),|(t,u,f),d|{
            (t+d.total_space(),u+(d.total_space()-d.available_space()),f+d.available_space())
        });
        let stor_pct=if stor_total>0{stor_used as f32/stor_total as f32*100.0}else{0.0};
        let (ts,tr):(u64,u64)=networks.iter().fold((0,0),|(s,r),(_,n)|(s+n.total_transmitted(),r+n.total_received()));
        let now=Instant::now(); let elapsed=now.duration_since(last_t).as_secs_f64();
        let speed_up=if elapsed>0.0{(ts.saturating_sub(last_sent)) as f64/elapsed}else{0.0};
        let speed_down=if elapsed>0.0{(tr.saturating_sub(last_recv)) as f64/elapsed}else{0.0};
        last_sent=ts; last_recv=tr; last_t=now;
        let ip=networks.iter().flat_map(|(_,n)|n.ip_networks()).find(|ip|!ip.addr.is_loopback()&&ip.addr.is_ipv4()).map(|ip|ip.addr.to_string()).unwrap_or_else(||"N/A".into());
        let mut procs:Vec<ProcessInfo>=sys.processes().values().map(|p:&Process|ProcessInfo{pid:p.pid().as_u32(),name:p.name().to_string_lossy().into_owned(),cpu:p.cpu_usage(),mem:p.memory() as f32/total as f32*100.0,status:format!("{:?}",p.status())}).collect();
        procs.sort_by(|a,b|b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal)); procs.truncate(20);
        let uptime_secs=System::uptime();
        let battery=read_battery();
        let device=read_device_info();
        let mut d=data.lock().unwrap();
        d.cpu=CpuData{percent:cpu_pct,per_core,count,model,freq_mhz};
        d.memory=MemData{total,used,available,percent:mem_pct,swap_total,swap_used,swap_percent:swap_pct};
        d.storage=StorageData{total:stor_total,used:stor_used,free:stor_free,percent:stor_pct};
        d.network=NetData{ip,bytes_sent:ts,bytes_recv:tr,speed_up,speed_down};
        d.processes=procs; d.battery=battery; d.uptime_secs=uptime_secs;
        if d.device.kernel.is_empty() { d.device=device; }
        drop(d);
        thread::sleep(Duration::from_millis(500));
    }
}

fn read_battery()->BatteryData {
    if let Ok(out)=std::process::Command::new("termux-battery-status").output() {
        if let Ok(s)=std::str::from_utf8(&out.stdout) {
            fn extract<'a>(s:&'a str,key:&str)->Option<&'a str>{
                let k=format!("\"{}\"",key); let pos=s.find(&k)?;
                let rest=&s[pos+k.len()..]; let colon=rest.find(':')? +1;
                let val=rest[colon..].trim();
                if val.starts_with('"'){let end=val[1..].find('"')?;Some(&val[1..=end])}
                else{let end=val.find(|c:char|c==','||c=='}').unwrap_or(val.len());Some(val[..end].trim())}
            }
            return BatteryData {
                percentage:extract(s,"percentage").and_then(|v|v.parse().ok()).unwrap_or(0),
                status:extract(s,"status").unwrap_or("Unknown").to_string(),
                health:extract(s,"health").unwrap_or("Unknown").to_string(),
                temperature:extract(s,"temperature").and_then(|v|v.parse().ok()).unwrap_or(0.0),
                plugged:extract(s,"plugged").unwrap_or("Unknown").to_string(),
                current_ua:extract(s,"current").and_then(|v|v.parse().ok()).unwrap_or(0),
                time_remaining:"N/A".into(),
            };
        }
    }
    BatteryData{percentage:0,status:"N/A".into(),health:"N/A".into(),temperature:0.0,plugged:"N/A".into(),current_ua:0,time_remaining:"N/A".into()}
}

fn read_device_info()->DeviceInfo {
    let gp=|k:&str|std::process::Command::new("getprop").arg(k).output().ok().and_then(|o|String::from_utf8(o.stdout).ok()).unwrap_or_default().trim().to_string();
    let uname=std::process::Command::new("uname").arg("-r").output().ok().and_then(|o|String::from_utf8(o.stdout).ok()).unwrap_or_default().trim().to_string();
    DeviceInfo{model:gp("ro.product.model"),android:gp("ro.build.version.release"),arch:gp("ro.product.cpu.abi"),manufacturer:gp("ro.product.manufacturer"),kernel:uname}
}

fn fmt_bytes(b:u64)->String { const U:&[&str]=&["B","KB","MB","GB","TB"]; let mut v=b as f64; let mut i=0; while v>=1024.0&&i<U.len()-1{v/=1024.0;i+=1;} format!("{:.1}{}",v,U[i]) }
fn fmt_speed(b:f64)->String { format!("{}/s",fmt_bytes(b as u64)) }
fn fmt_uptime(s:u64)->String { let d=s/86400;let h=(s%86400)/3600;let m=(s%3600)/60; if d>0{format!("{}d {}h {}m",d,h,m)}else if h>0{format!("{}h {}m",h,m)}else{format!("{}m",m)} }
fn gc(p:f64)->Color { if p>=90.0{Color::Red}else if p>=70.0{Color::Yellow}else{Color::Green} }
fn gauge<'a>(label:&'a str,pct:f64)->Gauge<'a> { Gauge::default().block(Block::default().title(Span::styled(label,Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))).gauge_style(Style::default().fg(gc(pct))).percent(pct.clamp(0.0,100.0) as u16).label(format!("{:.1}%",pct)) }

fn render(f:&mut Frame, app:&mut App) {
    let data=app.data.lock().unwrap().clone();
    let root=Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3),Constraint::Min(0)]).split(f.area());
    render_header(f,root[0],&data);
    let body=Layout::default().direction(Direction::Horizontal).constraints([Constraint::Length(16),Constraint::Min(0)]).split(root[1]);
    render_sidebar(f,body[0],app.tab);
    match app.tab {
        Tab::Overview=>render_overview(f,body[1],&data),
        Tab::Cpu=>render_cpu(f,body[1],&data),
        Tab::Memory=>render_memory(f,body[1],&data),
        Tab::Storage=>render_storage(f,body[1],&data,&mut app.explorer),
        Tab::Battery=>render_battery(f,body[1],&data),
        Tab::Network=>render_network(f,body[1],&data),
        Tab::Processes=>render_processes(f,body[1],&data),
        Tab::Settings=>render_settings(f,body[1],&app.settings),
    }
}

fn render_header(f:&mut Frame,area:Rect,data:&SystemData) {
    let now=Local::now();
    let line=Line::from(vec![
        Span::styled("🚀 TmxMon",Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  │  "),Span::styled(now.format("📅 %Y-%m-%d").to_string(),Style::default().fg(Color::Blue)),
        Span::raw("  "),Span::styled(now.format("🕐 %H:%M:%S").to_string(),Style::default().fg(Color::Green)),
        Span::raw("  │  "),Span::styled(format!("⏱ {}",fmt_uptime(data.uptime_secs)),Style::default().fg(Color::Yellow)),
        Span::raw("  │  "),Span::styled("↑↓:tabs  q:quit",Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(line).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan))).alignment(Alignment::Center),area);
}

fn render_sidebar(f:&mut Frame,area:Rect,current:Tab) {
    let items:Vec<ListItem>=Tab::ALL.iter().map(|&t|{
        let s=format!("  {}",t.label());
        if t==current { ListItem::new(s).style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)) }
        else { ListItem::new(s).style(Style::default().fg(Color::DarkGray)) }
    }).collect();
    f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)).title("Tabs")),area);
}

fn render_overview(f:&mut Frame,area:Rect,data:&SystemData) {
    let b=Block::default().borders(Borders::ALL).title(Span::styled(" 📊 Overview ",Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let inner=b.inner(area); f.render_widget(b,area);
    let rows=Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3),Constraint::Length(3),Constraint::Length(3),Constraint::Length(3),Constraint::Length(3),Constraint::Min(0)]).split(inner);
    f.render_widget(gauge(" CPU",data.cpu.percent as f64),rows[0]);
    f.render_widget(gauge(" Memory",data.memory.percent as f64),rows[1]);
    f.render_widget(gauge(" Storage",data.storage.percent as f64),rows[2]);
    f.render_widget(gauge(" Battery",data.battery.percentage as f64),rows[3]);
    let cols=Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(50),Constraint::Percentage(50)]).split(rows[4]);
    f.render_widget(Paragraph::new(Line::from(vec![Span::styled("↑ ",Style::default().fg(Color::Green)),Span::raw(fmt_speed(data.network.speed_up)),Span::raw("  "),Span::styled("↓ ",Style::default().fg(Color::Blue)),Span::raw(fmt_speed(data.network.speed_down))])).block(Block::default().title(" 🌐 Network").borders(Borders::ALL)),cols[0]);
    f.render_widget(Paragraph::new(Line::from(vec![Span::styled(format!("{} {}",&data.device.manufacturer,&data.device.model),Style::default().fg(Color::White))])).block(Block::default().title(" 📱 Device").borders(Borders::ALL)),cols[1]);
}

fn render_cpu(f:&mut Frame,area:Rect,data:&SystemData) {
    let b=Block::default().borders(Borders::ALL).title(Span::styled(" 💻 CPU ",Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let inner=b.inner(area); f.render_widget(b,area);
    let n=data.cpu.per_core.len().min(8);
    let mut c=vec![Constraint::Length(3)];
    for _ in 0..n { c.push(Constraint::Length(2)); }
    c.push(Constraint::Length(3)); c.push(Constraint::Min(0));
    let rows=Layout::default().direction(Direction::Vertical).constraints(c).split(inner);
    f.render_widget(gauge(" Overall",data.cpu.percent as f64),rows[0]);
    for (i,&p) in data.cpu.per_core.iter().take(8).enumerate() {
        f.render_widget(Gauge::default().gauge_style(Style::default().fg(gc(p as f64))).percent(p.clamp(0.0,100.0) as u16).label(format!("Core {} {:.1}%",i,p)),rows[1+i]);
    }
    let avg=if data.cpu.freq_mhz.is_empty(){"N/A".into()}else{format!("{} MHz",data.cpu.freq_mhz.iter().sum::<u64>()/data.cpu.freq_mhz.len() as u64)};
    let max=data.cpu.freq_mhz.iter().max().map(|f|format!("{} MHz",f)).unwrap_or_else(||"N/A".into());
    let info=vec![
        Line::from(vec![Span::styled("Model: ",Style::default().fg(Color::Cyan)),Span::raw(&data.cpu.model)]),
        Line::from(vec![Span::styled("Cores: ",Style::default().fg(Color::Cyan)),Span::raw(data.cpu.count.to_string()),Span::raw("  Avg: "),Span::raw(avg),Span::raw("  Max: "),Span::raw(max)]),
    ];
    f.render_widget(Paragraph::new(info).block(Block::default().title(" Info").borders(Borders::TOP)),rows[1+n]);
}

fn render_memory(f:&mut Frame,area:Rect,data:&SystemData) {
    let b=Block::default().borders(Borders::ALL).title(Span::styled(" 🧠 Memory ",Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let inner=b.inner(area); f.render_widget(b,area);
    let rows=Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3),Constraint::Length(3),Constraint::Length(6),Constraint::Min(0)]).split(inner);
    f.render_widget(gauge(" RAM",data.memory.percent as f64),rows[0]);
    f.render_widget(gauge(" Swap",data.memory.swap_percent as f64),rows[1]);
    let stats=vec![
        Line::from(vec![Span::styled("Total:     ",Style::default().fg(Color::Cyan)),Span::raw(fmt_bytes(data.memory.total))]),
        Line::from(vec![Span::styled("Used:      ",Style::default().fg(Color::Cyan)),Span::raw(fmt_bytes(data.memory.used))]),
        Line::from(vec![Span::styled("Available: ",Style::default().fg(Color::Cyan)),Span::raw(fmt_bytes(data.memory.available))]),
        Line::from(vec![Span::styled("Swap:      ",Style::default().fg(Color::Cyan)),Span::raw(format!("{} / {}",fmt_bytes(data.memory.swap_used),fmt_bytes(data.memory.swap_total)))]),
    ];
    f.render_widget(Paragraph::new(stats).block(Block::default().title(" Stats").borders(Borders::TOP)),rows[2]);
}

fn render_storage(f:&mut Frame,area:Rect,data:&SystemData,ex:&mut FileExplorer) {
    if !ex.focused {
        let b=Block::default().borders(Borders::ALL).title(Span::styled(" 💾 Storage ",Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
        let inner=b.inner(area); f.render_widget(b,area);
        let rows=Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3),Constraint::Length(6),Constraint::Min(0)]).split(inner);
        f.render_widget(gauge(" Storage",data.storage.percent as f64),rows[0]);
        let stats=vec![
            Line::from(vec![Span::styled("Total: ",Style::default().fg(Color::Cyan)),Span::raw(fmt_bytes(data.storage.total))]),
            Line::from(vec![Span::styled("Used:  ",Style::default().fg(Color::Cyan)),Span::raw(fmt_bytes(data.storage.used))]),
            Line::from(vec![Span::styled("Free:  ",Style::default().fg(Color::Cyan)),Span::raw(fmt_bytes(data.storage.free))]),
            Line::from(vec![Span::styled("Tip:   ",Style::default().fg(Color::DarkGray)),Span::styled("Press Enter for file explorer",Style::default().fg(Color::DarkGray))]),
        ];
        f.render_widget(Paragraph::new(stats).block(Block::default().title(" Stats").borders(Borders::TOP)),rows[1]);
    } else {
        let b=Block::default().borders(Borders::ALL).title(Span::styled(" 📂 File Explorer ",Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
        let inner=b.inner(area); f.render_widget(b,area);
        let rows=Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(1),Constraint::Min(0),Constraint::Length(1)]).split(inner);
        f.render_widget(Paragraph::new(Line::from(vec![Span::styled(ex.current_path.to_string_lossy().to_string(),Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))])),rows[0]);
        let max=rows[1].height as usize; let offset=ex.offset; let sel=ex.selected;
        let slice=ex.visible(max);
        let items:Vec<ListItem>=slice.iter().enumerate().map(|(i,e)|{
            let actual=offset+i;
            let sz=if e.is_dir{if e.name!=".." {format!(" ({} items)",e.count)}else{String::new()}}else{format!(" ({})",fmt_bytes(e.size))};
            let line=Line::from(vec![
                Span::raw(if actual==sel{"▶ "}else{"  "}),
                Span::raw(&e.name),
                Span::styled(sz,Style::default().fg(Color::DarkGray)),
            ]);
            if actual==sel { ListItem::new(line).style(Style::default().fg(Color::Black).bg(Color::Cyan)) }
            else if e.is_dir { ListItem::new(line).style(Style::default().fg(Color::Blue)) }
            else { ListItem::new(line) }
        }).collect();
        f.render_widget(List::new(items),rows[1]);
        f.render_widget(Paragraph::new(Span::styled("↑↓:Navigate  Enter:Open  Esc:Back",Style::default().fg(Color::DarkGray))),rows[2]);
    }
}

fn render_battery(f:&mut Frame,area:Rect,data:&SystemData) {
    let bat=&data.battery;
    let b=Block::default().borders(Borders::ALL).title(Span::styled(" 🔋 Battery ",Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let inner=b.inner(area); f.render_widget(b,area);
    let rows=Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3),Constraint::Min(0)]).split(inner);
    f.render_widget(gauge(" Charge Level",bat.percentage as f64),rows[0]);
    let lines=vec![
        Line::from(vec![Span::styled("Status:      ",Style::default().fg(Color::Cyan)),Span::raw(&bat.status)]),
        Line::from(vec![Span::styled("Health:      ",Style::default().fg(Color::Cyan)),Span::raw(&bat.health)]),
        Line::from(vec![Span::styled("Temperature: ",Style::default().fg(Color::Cyan)),Span::raw(format!("{:.1}°C",bat.temperature))]),
        Line::from(vec![Span::styled("Plugged:     ",Style::default().fg(Color::Cyan)),Span::raw(&bat.plugged)]),
        Line::from(vec![Span::styled("Current:     ",Style::default().fg(Color::Cyan)),Span::raw(format!("{} µA",bat.current_ua))]),
    ];
    f.render_widget(Paragraph::new(lines).block(Block::default().title(" Details").borders(Borders::TOP)),rows[1]);
}

fn render_network(f:&mut Frame,area:Rect,data:&SystemData) {
    let net=&data.network;
    let b=Block::default().borders(Borders::ALL).title(Span::styled(" 🌐 Network ",Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let inner=b.inner(area); f.render_widget(b,area);
    let rows=Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(4),Constraint::Min(0)]).split(inner);
    let speed=vec![
        Line::from(vec![Span::styled("↑ Upload:   ",Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),Span::styled(fmt_speed(net.speed_up),Style::default().fg(Color::Green))]),
        Line::from(vec![Span::styled("↓ Download: ",Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),Span::styled(fmt_speed(net.speed_down),Style::default().fg(Color::Blue))]),
    ];
    f.render_widget(Paragraph::new(speed).block(Block::default().title(" Speed").borders(Borders::ALL)),rows[0]);
    let stats=vec![
        Line::from(vec![Span::styled("IP (v4):        ",Style::default().fg(Color::Cyan)),Span::raw(&net.ip)]),
        Line::from(vec![Span::styled("Total Sent:     ",Style::default().fg(Color::Cyan)),Span::raw(fmt_bytes(net.bytes_sent))]),
        Line::from(vec![Span::styled("Total Received: ",Style::default().fg(Color::Cyan)),Span::raw(fmt_bytes(net.bytes_recv))]),
    ];
    f.render_widget(Paragraph::new(stats).block(Block::default().title(" Stats").borders(Borders::TOP)),rows[1]);
}

fn render_processes(f:&mut Frame,area:Rect,data:&SystemData) {
    let b=Block::default().borders(Borders::ALL).title(Span::styled(" ⚙️  Processes ",Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let header=Row::new(["PID","Name","CPU%","MEM%","Status"].iter().map(|h|ratatui::widgets::Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))).height(1).bottom_margin(1);
    let rows:Vec<Row>=data.processes.iter().map(|p|Row::new(vec![
        ratatui::widgets::Cell::from(p.pid.to_string()),
        ratatui::widgets::Cell::from(p.name.chars().take(28).collect::<String>()),
        ratatui::widgets::Cell::from(format!("{:.1}",p.cpu)).style(Style::default().fg(Color::Yellow)),
        ratatui::widgets::Cell::from(format!("{:.1}",p.mem)).style(Style::default().fg(Color::Green)),
        ratatui::widgets::Cell::from(p.status.chars().take(10).collect::<String>()).style(Style::default().fg(Color::DarkGray)),
    ])).collect();
    f.render_widget(Table::new(rows,[Constraint::Length(8),Constraint::Min(20),Constraint::Length(8),Constraint::Length(8),Constraint::Length(12)]).header(header).block(b),area);
}

fn render_settings(f:&mut Frame,area:Rect,s:&Settings) {
    let b=Block::default().borders(Borders::ALL).title(Span::styled(" ⚙️  Settings ",Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let inner=b.inner(area); f.render_widget(b,area);
    let parts=Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(0),Constraint::Length(2)]).split(inner);
    let entries=[("Refresh Rate",format!("{} ms",s.refresh_ms)),("Battery Capacity",format!("{} mAh",s.battery_mah))];
    let rows:Vec<Row>=entries.iter().enumerate().map(|(i,(name,val))|{
        let style=if i==s.selected{Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)}else{Style::default()};
        Row::new(vec![
            ratatui::widgets::Cell::from(if i==s.selected{"▶"}else{" "}),
            ratatui::widgets::Cell::from(*name).style(Style::default().fg(Color::Cyan)),
            ratatui::widgets::Cell::from(val.clone()),
        ]).style(style)
    }).collect();
    f.render_widget(Table::new(rows,[Constraint::Length(2),Constraint::Length(25),Constraint::Min(20)]).header(Row::new(["","Setting","Value"]).style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)).bottom_margin(1)),parts[0]);
    f.render_widget(Paragraph::new(Line::from(vec![Span::styled("↑↓:Navigate  ",Style::default().fg(Color::DarkGray)),Span::styled("←→:Adjust  ",Style::default().fg(Color::DarkGray)),Span::styled("r:Reset",Style::default().fg(Color::Yellow))])),parts[1]);
}

fn handle_input(app:&mut App)->Result<()> {
    if event::poll(Duration::from_millis(50))? {
      if let Event::Key(key)=event::read()? {
            if key.kind!=KeyEventKind::Press { return Ok(()); }

            // 1. Handle global keys FIRST so they are never blocked
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => { app.running = false; return Ok(()); }
                KeyCode::Tab => { app.next(); return Ok(()); }
                KeyCode::BackTab => { app.prev(); return Ok(()); }
                _ => {}
            }

            // 2. Handle File Explorer overrides
            if key.code==KeyCode::Esc { if app.explorer.focused { app.explorer.focused=false; return Ok(()); } }
            if app.tab==Tab::Storage&&app.explorer.focused {
                match key.code { KeyCode::Up=>app.explorer.up(), KeyCode::Down=>app.explorer.down(), KeyCode::Enter=>app.explorer.enter(), _=>{} }
                return Ok(());
            }

            // 3. Handle Settings overrides
            if app.tab==Tab::Settings {
                match key.code {
                    KeyCode::Up=>{ if app.settings.selected>0{app.settings.selected-=1;} }
                    KeyCode::Down=>{ if app.settings.selected<1{app.settings.selected+=1;} }
                    KeyCode::Right=>match app.settings.selected { 0=>app.settings.refresh_ms=(app.settings.refresh_ms+100).min(2000), 1=>app.settings.battery_mah=(app.settings.battery_mah+500).min(10000), _=>{} }
                    KeyCode::Left=>match app.settings.selected { 0=>app.settings.refresh_ms=app.settings.refresh_ms.saturating_sub(100).max(100), 1=>app.settings.battery_mah=(app.settings.battery_mah-500).max(1000), _=>{} }
                    KeyCode::Char('r')|KeyCode::Char('R')=>app.settings=Settings::default(),
                    _=>{}
                }
                return Ok(());
            }

            // 4. Default navigation for standard tabs
            match key.code {
                KeyCode::Up=>app.prev(), KeyCode::Down=>app.next(),
                KeyCode::Enter=>{ if app.tab==Tab::Storage { app.explorer.focused=true; } }
                _=>{}
            }
        }
    }
    Ok(())
}




fn main()->Result<()> {
    enable_raw_mode()?;
    let mut stdout=io::stdout();
    execute!(stdout,EnterAlternateScreen,EnableMouseCapture)?;
    let backend=CrosstermBackend::new(stdout);
    let mut terminal=Terminal::new(backend)?;
    let mut app=App::new();
    let data_clone=Arc::clone(&app.data);
    thread::spawn(move||collect_loop(data_clone));
    while app.running {
        terminal.draw(|f|render(f,&mut app))?;
        handle_input(&mut app)?;
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(),LeaveAlternateScreen,DisableMouseCapture)?;
    terminal.show_cursor()?;
    println!("✨ TmxMon closed!");
    Ok(())
}
