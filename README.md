<div align="center">

# rfdisk

**面向 Linux 的 refresh-first、类 cfdisk 分区表编辑工具。**

[简体中文](README.md) | [English](README.en.md)

![license](https://img.shields.io/badge/license-MIT-green)
![rust](https://img.shields.io/badge/rust-2021-orange)
![platform](https://img.shields.io/badge/platform-Linux-blue)
![status](https://img.shields.io/badge/status-alpha-yellow)

</div>

rfdisk 是一个用 Rust 编写的 Linux 终端分区表工具。它的设计灵感来自 `cfdisk`，但更强调 refresh-first 工作流，适合在工具运行过程中识别新插入或热插拔的磁盘。

当前版本仍是 alpha / 测试版。请优先在 Linux 虚拟机或可丢弃测试磁盘上使用。

## 已测试平台

当前 alpha 已经手动测试通过：

1. Ubuntu
2. Debian
3. CentOS 9 / 兼容的 RHEL 系发行版环境

Windows 仅作为本仓库的开发环境。

## 安装

可以用一行命令安装 rfdisk：

```sh
curl -fsSL https://raw.githubusercontent.com/EasonLin-X/rfdisk/main/install.sh | sh
```

更稳妥的手动形式：

```sh
curl -fsSL -o install.sh https://raw.githubusercontent.com/EasonLin-X/rfdisk/main/install.sh
sh install.sh
```

安装脚本会根据当前 Linux 架构下载对应 Release 资源，并安装到：

```text
/usr/local/bin/rfdisk
```

预期 Release 文件名：

```text
rfdisk-linux-x86_64.tar.gz
rfdisk-linux-aarch64.tar.gz
```

也可以覆盖仓库、版本或安装目录：

```sh
RFDISK_REPO=EasonLin-X/rfdisk sh install.sh
RFDISK_VERSION=v0.1.0 sh install.sh
RFDISK_INSTALL_DIR="$HOME/.local/bin" sh install.sh
```

## 功能

1. 在终端 UI 中列出物理磁盘。
2. 通过 `sfdisk`、`lsblk`、`blkid`、`/sys`、`udev` 等 Linux 信息源读取真实分区表。
3. 支持 GPT 和 MBR 分区表编辑。
4. 支持 `New`、`Delete`、分区级 `Type`、`Commit`、`Cancel` 和 `Write`。
5. 支持真实 free space 计算，包括中间空洞。
6. 所有编辑先进入内存草稿，用户确认后才写盘。
7. 写盘前显示 preview / risk 摘要。
8. 对系统保护盘、已挂载分区、swap 分区提供基础安全检查。
9. 使用原生 Linux 写盘链路：

```text
sfdisk -> partprobe/blockdev --rereadpt -> udevadm settle -> refresh
```

## 刷新 / 热插拔

按 `R` 触发深度刷新。

对于 SCSI 风格磁盘，rfdisk 当前会通过下面的路径把非保护 `sd*` 设备从内核设备树中移除：

```text
/sys/block/<disk>/device/delete
```

然后通过下面的路径触发主机扫描：

```text
/sys/class/scsi_host/host*/scan
```

这可以帮助工具在不重启的情况下识别新插入磁盘。建议在 Linux 上使用 `sudo` 运行。

## 命令行

```sh
rfdisk --help
rfdisk --version
rfdisk --lang en
rfdisk --lang zh-CN
```

支持语言：

1. 简体中文: `--lang zh-CN`
2. English: `--lang en`

使用 `--lang` 后，rfdisk 会保存这次语言选择，下次启动不需要重复输入。如果没有保存过语言，rfdisk 会尝试跟随系统默认语言。

## 日志

写盘日志保存到：

```text
/var/log/rfdisk/<timestamp>.log
```

请使用足够权限运行，让 rfdisk 能够创建并写入该目录。

## 当前安全边界

rfdisk alpha 版本专注于分区表编辑。

它暂时不承诺：

1. 无损 resize 或 move。
2. 完整 LVM / dm-crypt / RAID / multipath 安全检测。
3. 稳定的文件系统格式化或挂载管理工作流。
4. `wipefs` 或整盘清空。
5. 生产级恢复能力。

测试时请使用可丢弃磁盘。

## 构建

```sh
cargo build
```

在 Linux 上运行：

```sh
sudo ./target/debug/rfdisk
```

## 反馈

欢迎提交 issue、bug 报告、建议和真实 Linux 测试结果。

如果发现问题，请尽量提供：

1. Linux 发行版和内核版本。
2. util-linux 版本。
3. 磁盘类型和 VM / 硬件环境。
4. `/var/log/rfdisk/<timestamp>.log` 中的 rfdisk 日志。
5. 相关的 `lsblk`、`sfdisk --json` 或 `blkid` 输出。

## License

MIT License. See [LICENSE](LICENSE).
