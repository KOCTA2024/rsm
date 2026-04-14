use std::io;
use std::fs;
use std::time::Duration;

fn main(){
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    
    let mut sensors_list: Vec<String> = Vec::new();
    const CPU_TABLE_HEADER: [&str; 3] = ["CPU: Frequency (MHz)", "Temperature (°C)", "Power (W)"];
    const GPU_TABLE_HEADER: [&str; 5] = ["GPU: Fan Speed (RPM)", "GPU Clock (MHz)", "Memory Clock (MHz)", "GPU Temp (°C)", "GPU Power (W)"];
    dev_sensors(&mut sensors_list);
    let resmon_logo = r#"
    ____           __  ___           
   / __ \___  ____/  |/  /___  ____  
  / /_/ / _ \/ ___/ /|_/ / __ \/ __ \ 
 / _, _/  __(__  ) /  / / /_/ / / / / 
/_/ |_|\___/____/_/  /_/\____/_/ /_/  
 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    "#;

    println!("{}", resmon_logo);

    let mut input = String::new();
    println!("Hello! Type a character: ");

    println!("c: cpu; m: memory; d: disk; g: gpu; a: all; q: quit");
    println!("\n(basic:all)");

    io::stdin().read_line(&mut input).expect("Failed to read line");

    let trimmed = input.trim();


    let final_input = if trimmed.is_empty() {
        "a"
    } else {
        trimmed
    };

    if final_input == "q"{
    println!("exit...");
        return;
    }

    if final_input == "a"{
        loop {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!("{:?}", GPU_TABLE_HEADER);
            println!("{:?}", parse_gpu(&sensors_list));
            println!("{:?}", CPU_TABLE_HEADER);
            println!("{:?}", parse_cpu());
            std::thread::sleep(Duration::from_secs(1));
        }
        }
    else if final_input == "g"{
        loop {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!("{:?}", GPU_TABLE_HEADER);
            println!("{:?}", parse_gpu(&sensors_list));
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    else if final_input == "c"{
        loop {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!("{:?}", CPU_TABLE_HEADER);
            println!("{:?}", parse_cpu());
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    println!("You entered: {}", final_input);


    println!("{:?}", sensors_list);
    
    
}

fn dev_sensors(sensors_list: &mut Vec<String>){
    let main_path: &str = "/sys/class/hwmon";
    for path in fs::read_dir(main_path).expect("failed to read sensors dir"){
        let entry = path.expect("failed to read sensor entry");
        let p = entry.path();
        let name = fs::read_to_string(p.join("name")).unwrap_or_else(|_| "unknown".to_string());
        println!("Sensor: {}", name.trim());
        let folder_name = entry.file_name().to_string_lossy().to_string();
        sensors_list.push(folder_name);
    }
    
}

fn parse_gpu(sensors_list: &Vec<String>) -> [i32; 5]{
    let mut gpu_stats: [i32; 5] = [0; 5];
    let main_path: &str = "/sys/class/hwmon";
    let mut i: usize = 0;
    for sensor in sensors_list {
        let name_path = format!("{}/{}/name", main_path, sensor);
        let driver_name = fs::read_to_string(name_path).unwrap_or_default();

        if driver_name.trim() == "amdgpu" {
            let paths: [&str; 5] = ["fan1_input", "freq1_input", "freq2_input", "temp1_input", "power1_input"];
            for path in paths {
                let full_path = format!("{}/{}/{}", main_path, sensor, path);
                let mut val: usize = fs::read_to_string(full_path).unwrap_or_else(|_| "unknown".to_string()).trim().parse().unwrap_or(0);
                if path == "freq1_input" || path == "freq2_input" {
                    val /= 1000000;}
                if path == "temp1_input" {
                    val /= 1000;
                }
                if path == "power1_input" {
                    val /= 1000000;
                }
                gpu_stats[i] = val as i32;
                i += 1;
    }

}

    }
    gpu_stats
}

fn parse_cpu() -> [i32; 3]{
    // need to make temp path based on "coretemp" sensor, but for now just hardcoding it
    let cpu_freq_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq";
    let raw_cpu_power_path = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj";
    let cpu_temp_path = "/sys/class/hwmon/hwmon2/temp1_input";
    
    let cpu_freq = fs::read_to_string(cpu_freq_path).unwrap_or_else(|_| "unknown".to_string()).trim().parse::<i32>().unwrap_or(0) / 1000;
    let cpu_temp = fs::read_to_string(cpu_temp_path).unwrap_or_else(|_| "unknown".to_string()).trim().parse::<i32>().unwrap_or(0) / 1000;
    
    let e1: u64 = fs::read_to_string(raw_cpu_power_path).unwrap_or_default().trim().parse().unwrap_or(0);
    let t1 = std::time::Instant::now();
    std::thread::sleep(Duration::from_millis(100));

    let e2: u64 = fs::read_to_string(raw_cpu_power_path).unwrap_or_default().trim().parse().unwrap_or(0);
    let t2 = std::time::Instant::now();
    
    let delta_e = e2 - e1; 
    let delta_t = t2.duration_since(t1).as_secs_f32();
    
    let cpu_power = (delta_e as f32 / delta_t) / 1000000.0;
    
    let cpu_stats = [cpu_freq, cpu_temp, cpu_power as i32];
    cpu_stats

}