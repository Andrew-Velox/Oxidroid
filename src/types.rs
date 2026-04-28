#[derive(Default, Clone)]
pub struct CpuData { pub percent: f32, pub per_core: Vec<f32>, pub count: usize, pub model: String, pub freq_mhz: Vec<u64> }

#[derive(Default, Clone)]
pub struct MemData { pub total: u64, pub used: u64, pub available: u64, pub percent: f32, pub swap_total: u64, pub swap_used: u64, pub swap_percent: f32 }

#[derive(Default, Clone)]
pub struct StorageData { pub total: u64, pub used: u64, pub free: u64, pub percent: f32 }

#[derive(Default, Clone)]
pub struct BatteryData { pub percentage: u8, pub status: String, pub health: String, pub temperature: f32, pub plugged: String, pub current_ua: i64, pub time_remaining: String }

#[derive(Default, Clone)]
pub struct NetData { pub ip: String, pub bytes_sent: u64, pub bytes_recv: u64, pub speed_up: f64, pub speed_down: f64 }

#[derive(Default, Clone)]
pub struct ProcessInfo { pub pid: u32, pub name: String, pub cpu: f32, pub mem: f32, pub status: String }

#[derive(Default, Clone)]
pub struct DeviceInfo { pub model: String, pub android: String, pub arch: String, pub manufacturer: String, pub kernel: String }

#[derive(Default, Clone)]
pub struct SystemData { pub cpu: CpuData, pub memory: MemData, pub storage: StorageData, pub battery: BatteryData, pub network: NetData, pub processes: Vec<ProcessInfo>, pub device: DeviceInfo, pub uptime_secs: u64 }