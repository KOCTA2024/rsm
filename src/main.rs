use std::io;
use std::fs;
use std::time::{Duration, Instant};

const COLOR_RESET: &str = "\x1b[0m";
const COLOR_BOLD: &str = "\x1b[1m";
const COLOR_CYAN: &str = "\x1b[36m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_YELLOW: &str = "\x1b[33m";
const COLOR_MAGENTA: &str = "\x1b[35m";
const COLOR_BLUE: &str = "\x1b[34m";

const PCI_IDS_PATH: &str = "/usr/share/hwdata/pci.ids";

struct CpuState {
    last_energy: u64,
    last_time: Instant,
}

fn main() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    
    let mut sensors_list: Vec<String> = Vec::new();
    dev_sensors(&mut sensors_list);
    
    
    let raw_cpu_power_path = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj";
    let mut cpu_state = CpuState {
        last_energy: fs::read_to_string(raw_cpu_power_path).unwrap_or_default().trim().parse().unwrap_or(0),
        last_time: Instant::now(),
    };

    let resmon_logo = r#"
    ____           __  ___           
   / __ \___  ____/  |/  /___  ____  
  / /_/ / _ \/ ___/ /|_/ / __ \/ __ \ 
 / _, _/  __(__  ) /  / / /_/ / / / / 
/_/ |_|\___/____/_/  /_/\____/_/ /_/  
 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    "#;
    println!("{}{}{}", COLOR_CYAN, resmon_logo, COLOR_RESET);

    let mut input = String::new();
    println!("{}{}Hello! Type a character:{}", COLOR_BOLD, COLOR_GREEN, COLOR_RESET);
    println!("{}c: cpu | m: memory | g: gpu | a: all | q: quit{}", COLOR_YELLOW, COLOR_RESET);

    io::stdin().read_line(&mut input).expect("Failed to read line");
    let trimmed = input.trim();
    let final_input = if trimmed.is_empty() { "a" } else { trimmed };

    if final_input == "q" {
        println!("{}exit...{}", COLOR_YELLOW, COLOR_RESET);
        return;
    }

    loop {
        let loop_start = Instant::now();
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        
        match final_input {
            "a" => {
                display_gpu(&sensors_list);
                println!();
                display_memory();
                println!();
                display_cpu(&sensors_list, &mut cpu_state);
            }
            "g" => display_gpu(&sensors_list),
            "c" => display_cpu(&sensors_list, &mut cpu_state),
            "m" => display_memory(),
            _ => break,
        }

        
        let elapsed = loop_start.elapsed();
        if elapsed < Duration::from_secs(1) {
            std::thread::sleep(Duration::from_secs(1) - elapsed);
        }
    }
}

fn display_gpu(sensors_list: &Vec<String>) {
    let sensor_path = find_driver_path("amdgpu", sensors_list);
    let stats = parse_gpu(&sensor_path);
    let gpu_name = parse_gpu_name(&sensor_path);
    print_table(
        &format!("GPU: {}", gpu_name),
        &["Fan Speed (RPM)", "Clock (MHz)", "Mem Clock (MHz)", "Temp (°C)", "Power (W)"],
        &[stats[0].to_string(), stats[1].to_string(), stats[2].to_string(), stats[3].to_string(), stats[4].to_string()],
        COLOR_MAGENTA,
    );
}

fn display_cpu(sensors_list: &Vec<String>, state: &mut CpuState) {
    let stats = parse_cpu(sensors_list, state);
    print_table(
        &format!("CPU: {}", parse_cpu_name()),
        &["Frequency (MHz)", "Temperature (°C)", "Power (W)"],
        &[stats[0].to_string(), stats[1].to_string(), stats[2].to_string()],
        COLOR_BLUE,
    );
}

fn display_memory() {
    let stats = parse_memory();
    print_table(
        "RAM",
        &["Total (MB)", "Available (MB)", "Usage (%)"],
        &[
            format!("{:.2}", stats[0]),
            format!("{:.2}", stats[1]),
            format!("{:.2}", stats[2]),
        ],
        COLOR_GREEN,
    );
}

fn print_table(title: &str, headers: &[&str], values: &[String], color: &str) {
    let col_width = 22;
    let total_width = headers.len() * col_width + (headers.len() + 1);
    
    print!("{}", color);
    println!("┏{}┓", "━".repeat(total_width - 2));
    println!("┃{:^width$}┃", format!(" {} ", title), width = total_width - 2);
    
    print!("┣");
    for i in 0..headers.len() {
        print!("{}", "━".repeat(col_width));
        if i < headers.len() - 1 { print!("┳"); }
    }
    println!("┫");
    
    print!("┃");
    for header in headers {
        print!("{:^width$}┃", header, width = col_width);
    }
    println!();
    
    print!("┣");
    for i in 0..headers.len() {
        print!("{}", "━".repeat(col_width));
        if i < headers.len() - 1 { print!("╋"); }
    }
    println!("┫");
    
    print!("┃");
    for value in values {
        print!("{:^width$}┃", value, width = col_width);
    }
    println!();
    
    print!("┗");
    for i in 0..headers.len() {
        print!("{}", "━".repeat(col_width));
        if i < headers.len() - 1 { print!("┻"); }
    }
    println!("┛{}", COLOR_RESET);
}

fn dev_sensors(sensors_list: &mut Vec<String>) {
    let main_path = "/sys/class/hwmon";
    if let Ok(entries) = fs::read_dir(main_path) {
        for entry in entries.flatten() {
            sensors_list.push(entry.file_name().to_string_lossy().to_string());
        }
    }
}

fn find_driver_path(target_driver: &str, sensors_list: &[String]) -> String {
    let main_path = "/sys/class/hwmon";
    for sn in sensors_list {
        let name_path = format!("{}/{}/name", main_path, sn);
        if let Ok(content) = fs::read_to_string(&name_path) {
            if content.trim() == target_driver {
                return format!("{}/{}", main_path, sn);
            }
        }
    }
    "not_found".to_string()
}

fn parse_gpu_name(sensor_path: &str) -> String {
    if sensor_path == "not_found" {
        return "GPU Not Found".to_string();
    }

    let vendor_path = format!("{}/device/vendor", sensor_path);
    let device_path = format!("{}/device/device", sensor_path);

    let vendor = fs::read_to_string(vendor_path)
        .unwrap_or_default()
        .trim()
        .trim_start_matches("0x")
        .to_lowercase();
    let device = fs::read_to_string(device_path)
        .unwrap_or_default()
        .trim()
        .trim_start_matches("0x")
        .to_lowercase();

    if vendor.is_empty() || device.is_empty() {
        return "Unknown AMD GPU".to_string();
    }

    if let Ok(content) = fs::read_to_string(PCI_IDS_PATH) {
        let mut target_vendor_found = false;
        for line in content.lines() {
            if line.starts_with(&vendor) {
                target_vendor_found = true;
                continue;
            }

            if target_vendor_found {
                if line.starts_with('\t') && line.trim_start().starts_with(&device) {
                    return line.splitn(2, "  ").nth(1).unwrap_or("Unknown").trim().to_string();
                }
                if !line.starts_with('\t') && !line.starts_with('#') && !line.is_empty() {
                    break;
                }
            }
        }
    }

    format!("AMD GPU ({}:{})", vendor, device)
}

fn parse_gpu(sensor_path: &str) -> [i32; 5] {
    let mut gpu_stats = [0; 5];
    if sensor_path == "not_found" {
        return gpu_stats;
    }

    let paths = ["fan1_input", "freq1_input", "freq2_input", "temp1_input", "power1_input"];
    
    for (i, path) in paths.iter().enumerate() {
        let full_path = format!("{}/{}", sensor_path, path);
        let mut val: u64 = fs::read_to_string(full_path)
            .unwrap_or_default()
            .trim()
            .parse()
            .unwrap_or(0);
            
        match *path {
            "freq1_input" | "freq2_input" => val /= 1_000_000,
            "temp1_input" => val /= 1_000,
            "power1_input" => val /= 1_000_000,
            _ => {}
        }
        gpu_stats[i] = val as i32;
    }
    gpu_stats
}

fn parse_cpu(sensors_list: &[String], state: &mut CpuState) -> [i32; 3] {
    let cpu_freq_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq";
    let raw_cpu_power_path = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj";
    let sensor = find_driver_path("coretemp", sensors_list);
    let cpu_temp_path = format!("{}/temp1_input", sensor);

    let freq = fs::read_to_string(cpu_freq_path).unwrap_or_default().trim().parse::<i32>().unwrap_or(0) / 1000;
    let temp = fs::read_to_string(cpu_temp_path).unwrap_or_default().trim().parse::<i32>().unwrap_or(0) / 1000;
    

    let current_energy: u64 = fs::read_to_string(raw_cpu_power_path).unwrap_or_default().trim().parse().unwrap_or(0);
    let current_time = Instant::now();
    
    let energy_diff = current_energy.saturating_sub(state.last_energy);
    let time_diff = current_time.duration_since(state.last_time).as_secs_f32();
    
    let power = if time_diff > 0.0 {
        (energy_diff as f32 / time_diff) / 1_000_000.0
    } else {
        0.0
    };

  
    state.last_energy = current_energy;
    state.last_time = current_time;

    [freq, temp, power as i32]
}

fn parse_cpu_name() -> String {
    let raw_str = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    for line in raw_str.lines() {
        if line.contains("model name") {
            return line.split(':').nth(1).unwrap_or("").trim().to_string();
        } 
    }
    "Unknown CPU".to_string()
}

fn parse_memory() -> [f64; 3] {
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total = 0.0;
    let mut avail = 0.0;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            total = line.split_whitespace().nth(1).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0) / 1024.0;
        }
        if line.starts_with("MemAvailable:") {
            avail = line.split_whitespace().nth(1).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0) / 1024.0;
        }
    }

    let usage = if total > 0.0 { (1.0 - avail / total) * 100.0 } else { 0.0 };
    [total, avail, usage]
}
