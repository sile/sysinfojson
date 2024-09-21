use std::collections::BTreeMap;

use clap::Parser;
use orfail::OrFail;
use sysinfo::{Components, Disks, Networks, System, Users};

#[derive(Parser)]
#[clap(version)]
struct Args {
    // enum system {includes}, process {pid}
}

fn main() -> orfail::Result<()> {
    let _args = Args::parse();

    let mut sys = System::new_all();
    let load_avg = System::load_average();
    let json = serde_json::json! ({
        "system": system(),
        "boot_time": System::boot_time(),
        "uptime": System::uptime(),
        "users": users(),
        "disks": disks(),
        "networks": networks(),
        "temperatures": temperatures(),
        "load_avg": {
            "one": load_avg.one,
            "five": load_avg.five,
            "fifteen": load_avg.fifteen,
        },
        "process_count": sys.processes().len(),
        "cpus": cpus(&mut sys),
        "cpu": {
            "physical_core_count": sys.physical_core_count(),
            "global_cpu_usage": sys.global_cpu_usage(),
        },
        "memory": memory(&sys),
    });
    println!("{}", serde_json::to_string(&json).or_fail()?);

    Ok(())
}

fn system() -> serde_json::Value {
    serde_json::json! ({
        "name": System::name(),
        "kernel_version": System::kernel_version(),
        "os_version": System::os_version(),
        "long_os_version": System::long_os_version(),
        "host_name": System::host_name(),
    })
}

fn users() -> BTreeMap<String, serde_json::Value> {
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

fn disks() -> BTreeMap<String, serde_json::Value> {
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

fn networks() -> BTreeMap<String, serde_json::Value> {
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

fn temperatures() -> BTreeMap<String, serde_json::Value> {
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

fn cpus(sys: &mut System) -> BTreeMap<String, serde_json::Value> {
    // TODO:
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_all();

    sys.cpus()
        .into_iter()
        .map(|cpu| {
            (
                cpu.name().to_string(),
                serde_json::json!({
                    "brand": cpu.brand(),
                    "vendor_id": cpu.vendor_id(),
                    "frequency": cpu.frequency(),
                    "cpu_usage": cpu.cpu_usage(),
                }),
            )
        })
        .collect()
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
