<div align="center">

# rfdisk

**A refresh-first cfdisk-like partition table editor for Linux.**

[Simplified Chinese](README.md) | [English](README.en.md)

![license](https://img.shields.io/badge/license-MIT-green)
![rust](https://img.shields.io/badge/rust-2021-orange)
![platform](https://img.shields.io/badge/platform-Linux-blue)
![status](https://img.shields.io/badge/status-alpha-yellow)

</div>

rfdisk is a Linux-first terminal disk partition tool written in Rust. It is inspired by `cfdisk`, but focuses on a refresh-first workflow for disks that may be hot-plugged while the tool is running.

This is an alpha/test release. Please test with Linux virtual machines or disposable disks first.

## Tested Platforms

The current alpha has been manually tested on:

1. Ubuntu
2. Debian
3. CentOS 9 / compatible RHEL-family environments

Windows is only used as a development environment for this repository.

## Install

friends can install rfdisk with one command:

```sh
curl -fsSL https://raw.githubusercontent.com/EasonLin-X/rfdisk/main/install.sh | sh
```

Safer manual form:

```sh
curl -fsSL -o install.sh https://raw.githubusercontent.com/EasonLin-X/rfdisk/main/install.sh
sh install.sh
```

The installer downloads the matching release asset for the current Linux architecture and installs it to:

```text
/usr/local/bin/rfdisk
```

Expected release asset names:

```text
rfdisk-linux-x86_64.tar.gz
rfdisk-linux-aarch64.tar.gz
```

You can override the repository, version, or install directory:

```sh
RFDISK_REPO=EasonLin-X/rfdisk sh install.sh
RFDISK_VERSION=v0.1.0 sh install.sh
RFDISK_INSTALL_DIR="$HOME/.local/bin" sh install.sh
```

## Features

1. Physical disk list in a terminal UI.
2. Real partition-table reading through Linux sources such as `sfdisk`, `lsblk`, `blkid`, `/sys`, and `udev`.
3. GPT and MBR partition-table editing.
4. `New`, `Delete`, per-partition `Type`, `Commit`, `Cancel`, and `Write`.
5. Real free-space calculation, including middle gaps.
6. Draft-based editing before writing to disk.
7. Preview/risk summary before writing.
8. Safety checks for protected system disks, mounted partitions, and swap partitions.
9. Native Linux write chain:

```text
sfdisk -> partprobe/blockdev --rereadpt -> udevadm settle -> refresh
```

## Refresh / Hotplug

Press `R` to trigger a deep refresh.

For SCSI-style disks, rfdisk removes non-protected `sd*` devices from the kernel device tree through:

```text
/sys/block/<disk>/device/delete
```

Then it triggers host scans through:

```text
/sys/class/scsi_host/host*/scan
```

This helps detect newly attached disks without restarting the tool. Run with `sudo` on Linux for best results.

## Command Line

```sh
rfdisk --help
rfdisk --version
rfdisk --lang en
rfdisk --lang zh-CN
```

Supported languages:

1. Simplified Chinese: `--lang zh-CN`
2. English: `--lang en`

If `--lang` is used, rfdisk saves the selection for future runs. Without a saved choice, it tries to follow the system locale.

## Logs

Write logs are stored in:

```text
/var/log/rfdisk/<timestamp>.log
```

Run with sufficient privileges so rfdisk can create and write this directory.

## Current Safety Boundary

rfdisk alpha focuses on partition-table editing.

It does not promise:

1. No-loss resize or move.
2. Full LVM/dm-crypt/RAID/multipath safety detection.
3. Stable filesystem formatting or mount-management workflow.
4. `wipefs` or whole-disk wipe.
5. Production-grade recovery behavior.

Use disposable disks while testing.

## Build

```sh
cargo build
```

Run on Linux:

```sh
sudo ./target/debug/rfdisk
```

## Feedback

Issues, bug reports, suggestions, and real Linux test results are welcome.

If something looks wrong, please include:

1. Linux distribution and kernel version.
2. util-linux version.
3. Disk type and VM/hardware environment.
4. The rfdisk log from `/var/log/rfdisk/<timestamp>.log`.
5. Output from `lsblk`, `sfdisk --json`, or `blkid` when relevant.

## License

MIT License. See [LICENSE](LICENSE).
