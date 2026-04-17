use std::io;
use std::fs;
use std::time::Duration;

const COLOR_RESET: &str = "\x1b[0m";
const COLOR_BOLD: &str = "\x1b[1m";
const COLOR_CYAN: &str = "\x1b[36m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_YELLOW: &str = "\x1b[33m";
const COLOR_MAGENTA: &str = "\x1b[35m";
const COLOR_BLUE: &str = "\x1b[34m";

fn main() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    
    let mut sensors_list: Vec<String> = Vec::new();
    dev_sensors(&mut sensors_list);
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
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        match final_input {
            "a" => {
                display_gpu(&sensors_list);
                println!();
                display_memory();
                println!();
                display_cpu(&sensors_list);
            }
            "g" => display_gpu(&sensors_list),
            "c" => display_cpu(&sensors_list),
            "m" => display_memory(),
            _ => break,
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn display_gpu(sensors_list: &Vec<String>) {
    let stats = parse_gpu(sensors_list);
    print_table(
        "GPU Status",
        &["Fan Speed (RPM)", "Clock (MHz)", "Mem Clock (MHz)", "Temp (°C)", "Power (W)"],
        &[stats[0].to_string(), stats[1].to_string(), stats[2].to_string(), stats[3].to_string(), stats[4].to_string()],
        COLOR_MAGENTA,
    );
}

fn display_cpu(sensors_list: &Vec<String>) {
    let stats = parse_cpu(sensors_list);
    print_table(
        "CPU Status",
        &["Frequency (MHz)", "Temperature (°C)", "Power (W)"],
        &[stats[0].to_string(), stats[1].to_string(), stats[2].to_string()],
        COLOR_BLUE,
    );
}

fn display_memory() {
    let stats = parse_memory();
    print_table(
        "Memory Status",
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

fn parse_gpu(sensors_list: &Vec<String>) -> [i32; 5] {
    let mut gpu_stats = [0; 5];
    let sensor = find_driver_path("amdgpu", sensors_list);
    let paths = ["fan1_input", "freq1_input", "freq2_input", "temp1_input", "power1_input"];
    
    for (i, path) in paths.iter().enumerate() {
        let full_path = format!("{}/{}", sensor, path);
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

fn parse_cpu(sensors_list: &Vec<String>) -> [i32; 3] {
    let cpu_freq_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq";
    let raw_cpu_power_path = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj";
    let sensor = find_driver_path("coretemp", sensors_list);
    let cpu_temp_path = format!("{}/temp1_input", sensor);

    let freq = fs::read_to_string(cpu_freq_path).unwrap_or_default().trim().parse::<i32>().unwrap_or(0) / 1000;
    let temp = fs::read_to_string(cpu_temp_path).unwrap_or_default().trim().parse::<i32>().unwrap_or(0) / 1000;
    
    let e1: u64 = fs::read_to_string(raw_cpu_power_path).unwrap_or_default().trim().parse().unwrap_or(0);
    let t1 = std::time::Instant::now();
    std::thread::sleep(Duration::from_millis(100));
    let e2: u64 = fs::read_to_string(raw_cpu_power_path).unwrap_or_default().trim().parse().unwrap_or(0);
    let t2 = std::time::Instant::now();
    
    let p = ((e2.saturating_sub(e1)) as f32 / t2.duration_since(t1).as_secs_f32()) / 1_000_000.0;
    [freq, temp, p as i32]
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