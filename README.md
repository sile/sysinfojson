sysinfojson
===========

[![sysinfojson](https://img.shields.io/crates/v/sysinfojson.svg)](https://crates.io/crates/sysinfojson)
[![Actions Status](https://github.com/sile/sysinfojson/workflows/CI/badge.svg)](https://github.com/sile/sysinfojson/actions)
![License](https://img.shields.io/crates/l/sysinfojson)

Command-line tool that displays system information collected using [`sysinfo`] crate in JSON format.

[`sysinfo`]: https://crates.io/crates/sysinfo

```console
// Install.
$ cargo install sysinfojson

// Print help.
$ sysinfojson -h
Command-line tool that displays system information collected using `sysinfo` crate in JSON format

Usage: sysinfojson <COMMAND>

Commands:
  system   Displays system information
  process  Displays process information
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

// Display system information (all).
$ sysinfojson system | jq . | head -10
{
  "cpu": {
    "cpus": [
      {
        "brand": "Apple M1 Max",
        "cpu_usage": 63.6363639831543,
        "frequency": 3228,
        "name": "1",
        "vendor_id": "Apple"
      },

// Display system information (memory only).
$ sysinfojson system memory | jq .
{
  "memory": {
    "available_memory": 38833061888,
    "total_memory": 68719476736,
    "total_swap": 0,
    "used_memory": 33337425920,
    "used_swap": 0
  }
}

// Display process information (PID: 0).
$ sysinfojson process 1 | jq .
{
  "cmd": [],
  "cpu_usage": 0,
  "cwd": null,
  "disk_usage": {
    "read_bytes": 0,
    "total_read_bytes": 0,
    "total_written_bytes": 0,
    "written_bytes": 0
  },
  "effective_group_id": null,
  "effective_user_id": null,
  "environ": [],
  "exe": "/sbin/launchd",
  "group_id": null,
  "memory": 0,
  "name": "launchd",
  "parent": null,
  "root": null,
  "run_time": 0,
  "session_id": 1,
  "start_time": 0,
  "status": "Unknown",
  "tasks": null,
  "thread_kind": null,
  "user_id": null,
  "virtual_memory": 0
}
```
