use std::{
    collections::{BTreeMap, HashSet},
    time::Duration,
};

use clap::Parser;
use sysinfo::{Components, Disks, Networks, Pid, ProcessRefreshKind, RefreshKind, System, Users};

#[derive(Debug, Clone, PartialEq, Eq, Hash, clap::ValueEnum)]
enum SystemCategory {
    Cpu,
    Disk,
    LoadAvg,
    Memory,
    Network,
    Process,
    System,
    Temperature,
    User,
}

#[derive(Parser)]
#[clap(version)]
enum Args {
    System {
        categories: Vec<SystemCategory>,

        #[clap(short='i', long, default_value_t = sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.as_millis() as u16)]
        cpu_update_interval_ms: u16,
    },
    Process {
        pid: u32,
    },
}

fn main() {
    let args = Args::parse();
    let output = match args {
        Args::System {
            categories,
            cpu_update_interval_ms,
        } => {
            let categories = HashSet::from_iter(categories.into_iter());
            collect_system_info(
                &categories,
                Duration::from_millis(cpu_update_interval_ms as u64),
            )
        }
        Args::Process { pid } => collect_process_info(pid),
    };
    println!("{output}");
}

fn collect_process_info(pid: u32) -> serde_json::Value {
    let sys = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );
    let Some(proc) = sys.process(Pid::from_u32(pid)) else {
        return serde_json::Value::Null;
    };

    serde_json::json!({
        "name": proc.name().to_string_lossy(),
        "cmd": proc.cmd().iter().map(|c| c.to_string_lossy()).collect::<Vec<_>>(),
        "exe": proc.exe(),
        "cwd": proc.cwd(),
        "root": proc.root(),
        "memory": proc.memory(),
        "virtual_memory": proc.virtual_memory(),
        "parent": proc.parent().map(|p| p.as_u32()),
        "session_id": proc.session_id().map(|p| p.as_u32()),
        "tasks": proc.tasks().map(|pids| pids.iter().map(|p| p.as_u32()).collect::<Vec<_>>()),
        "user_id": proc.user_id().map(|u| **u),
        "effective_user_id": proc.effective_user_id().map(|u| **u),
        "group_id": proc.group_id().map(|g| *g),
        "effective_group_id": proc.effective_group_id().map(|g| *g),
        "status": proc.status().to_string(),
        "start_time": proc.start_time(),
        "run_time": proc.run_time(),
        "cpu_usage": proc.cpu_usage(),
        "disk_usage": proc.disk_usage(),
        "thread_kind": proc.thread_kind(),
        "environ": proc.environ().iter().map(|e| e.to_string_lossy()).collect::<Vec<_>>(),
    })
}

fn collect_system_info(
    categories: &HashSet<SystemCategory>,
    cpu_update_interval: Duration,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();

    if categories.is_empty()
        || categories.contains(&SystemCategory::Process)
        || categories.contains(&SystemCategory::Cpu)
        || categories.contains(&SystemCategory::Memory)
    {
        let mut sys = System::new_all();
        if categories.is_empty() || categories.contains(&SystemCategory::Cpu) {
            std::thread::sleep(cpu_update_interval);
            sys.refresh_cpu_all();

            map.insert("cpu".to_owned(), cpu(&sys));
        }
        if categories.is_empty() || categories.contains(&SystemCategory::Process) {
            map.insert(
                "process".to_owned(),
                serde_json::json!( {"process_count": sys.processes().len()}),
            );
        }
        if categories.is_empty() || categories.contains(&SystemCategory::Memory) {
            map.insert("memory".to_owned(), memory(&sys));
        }
    }

    if categories.is_empty() || categories.contains(&SystemCategory::System) {
        map.insert("system".to_owned(), system());
    }
    if categories.is_empty() || categories.contains(&SystemCategory::User) {
        map.insert("user".to_owned(), user());
    }
    if categories.is_empty() || categories.contains(&SystemCategory::Disk) {
        map.insert("disk".to_owned(), disk());
    }
    if categories.is_empty() || categories.contains(&SystemCategory::Network) {
        map.insert("network".to_owned(), network());
    }
    if categories.is_empty() || categories.contains(&SystemCategory::Temperature) {
        map.insert("temperature".to_owned(), temperature());
    }
    if categories.is_empty() || categories.contains(&SystemCategory::LoadAvg) {
        map.insert("load_avg".to_owned(), load_avg());
    }

    serde_json::Value::Object(map)
}

fn load_avg() -> serde_json::Value {
    let load_avg = System::load_average();
    serde_json::json! ({
        "one": load_avg.one,
        "five": load_avg.five,
        "fifteen": load_avg.fifteen,
    })
}

fn system() -> serde_json::Value {
    serde_json::json! ({
        "name": System::name(),
        "kernel_version": System::kernel_version(),
        "os_version": System::os_version(),
        "long_os_version": System::long_os_version(),
        "host_name": System::host_name(),
        "boot_time": System::boot_time(),
        "cpu_arch": System::cpu_arch(),
        "distribution_id": System::distribution_id(),
        "uptime": System::uptime(),
    })
}

fn user() -> serde_json::Value {
    Users::new_with_refreshed_list()
        .into_iter()
        .map(|user| {
            let groups = user
                .groups()
                .into_iter()
                .map(|group| (group.name().to_string(), **group.id()))
                .collect::<BTreeMap<_, _>>();
            (
                user.name().to_string(),
                serde_json::json!({"groups": groups}),
            )
        })
        .collect()
}

fn disk() -> serde_json::Value {
    Disks::new_with_refreshed_list()
        .into_iter()
        .map(|disk| {
            (
                disk.mount_point().to_string_lossy().to_string(),
                serde_json::json!({
                    "name": disk.name().to_string_lossy().to_string(),
                    "kind": disk.kind().to_string(),
                    "file_system": disk.file_system().to_string_lossy().to_string(),
                    "total_space": disk.total_space(),
                    "available_space": disk.available_space(),
                    "is_removable": disk.is_removable(),
                }),
            )
        })
        .collect()
}

fn network() -> serde_json::Value {
    Networks::new_with_refreshed_list()
        .into_iter()
        .map(|(interface_name, data)| {
            (
                interface_name.to_string(),
                serde_json::json!({
                    "mac_address": data.mac_address().to_string(),
                    "ip_networks": data.ip_networks().iter().map(|i| i.to_string()).collect::<Vec<_>>(),
                    "received": data.received(),
                    "total_received": data.total_received(),
                    "transmitted": data.transmitted(),
                    "total_transmitted": data.total_transmitted(),
                    "packets_received": data.packets_received(),
                    "total_packets_received": data.total_packets_received(),
                    "packets_transmitted": data.packets_transmitted(),
                    "total_packets_transmitted": data.total_packets_transmitted(),
                    "errors_on_received": data.errors_on_received(),
                    "total_errors_on_received": data.total_errors_on_received(),
                    "errors_on_transmitted": data.errors_on_transmitted(),
                    "total_errors_on_transmitted": data.total_errors_on_transmitted(),
                }),
            )
        })
        .collect()
}

fn temperature() -> serde_json::Value {
    Components::new_with_refreshed_list()
        .into_iter()
        .map(|component| {
            (
                component.label().to_string(),
                serde_json::json!({
                    "temperature": component.temperature(),
                    "max": component.max(),
                    "critical": component.critical(),
                }),
            )
        })
        .collect()
}

fn cpu(sys: &System) -> serde_json::Value {
    serde_json::json! ({
        "physical_core_count": sys.physical_core_count(),
        "global_cpu_usage": sys.global_cpu_usage(),
        "cpus": sys.cpus()
            .into_iter()
            .map(|cpu| {
                serde_json::json!({
                    "name": cpu.name(),
                    "brand": cpu.brand(),
                    "vendor_id": cpu.vendor_id(),
                    "frequency": cpu.frequency(),
                    "cpu_usage": cpu.cpu_usage(),
                })
            })
            .collect::<Vec<_>>()
    })
}

fn memory(sys: &System) -> serde_json::Value {
    serde_json::json!({
        "total_memory": sys.total_memory(),
        "available_memory": sys.available_memory(),
        "used_memory": sys.used_memory(),
        "total_swap": sys.total_swap(),
        "used_swap": sys.used_swap(),
    })
}
