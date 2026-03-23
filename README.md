# ClashTUI

<div align="center">

![ClashTUI Logo](https://via.placeholder.com/150x150.png?text=ClashTUI) <!-- 这里预留Logo位置，如果没有可以删除 -->

**一个基于 Rust 的 Clash 终端控制工具**

[![GitHub release (latest by date)](https://img.shields.io/github/v/release/ChanceFlow/ClashTUI?style=flat-square)](https://github.com/ChanceFlow/ClashTUI/releases)
[![GitHub license](https://img.shields.io/github/license/ChanceFlow/ClashTUI?style=flat-square)](https://github.com/ChanceFlow/ClashTUI/blob/master/LICENSE)
[![GitHub issues](https://img.shields.io/github/issues/ChanceFlow/ClashTUI?style=flat-square)](https://github.com/ChanceFlow/ClashTUI/issues)
[![GitHub stars](https://img.shields.io/github/stars/ChanceFlow/ClashTUI?style=flat-square)](https://github.com/ChanceFlow/ClashTUI/stargazers)
![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange?style=flat-square)

[功能特性](#-功能特性) • [安装](#-安装) • [使用说明](#-使用说明) • [快捷键](#%EF%B8%8F-快捷键) • [贡献](#-贡献)

</div>

---

**ClashTUI** 是一个功能丰富、轻量级的终端用户界面（TUI）工具，专为管理 Clash 代理核心而设计。它利用 Rust 的高性能和安全性，结合 `ratatui` 库，为您提供流畅的终端操作体验。无论是在服务器环境还是日常桌面使用，ClashTUI 都能让您高效地掌控网络代理状态。

## ✨ 功能特性

- **🎨 美观的 TUI 界面**：基于 `ratatui` 构建，支持自适应布局，清晰展示各类信息。
- **🚀 代理管理**：
    - 浏览所有代理组及节点。
    - **实时延迟测试**：支持批量或单个节点测速，自动标记超时节点。
    - **快速切换**：一键切换代理节点。
- **📊 状态监控**：
    - **实时流量**：底部状态栏实时显示上传/下载速度。
    - **连接管理**：查看当前所有活动连接（源IP、目标域名、流量使用情况）。
    - **模式切换**：快速在 Rule、Global、Direct 模式间切换。
- **📜 规则查看**：浏览当前的路由规则配置。
- **⌨️ 键盘优先**：支持 Vim 风格快捷键（`h`, `j`, `k`, `l`），全键盘操作无障碍。
- **💻 CLI 模式**：除了 TUI，还提供命令行工具，方便脚本调用或快速查询状态。

## 📦 安装

### 🍎 macOS / 🐧 Linux (Homebrew)

如果您使用 Homebrew，可以通过我们的 Tap 轻松安装：

```bash
brew tap ChanceFlow/OpenClashTUI
brew install clashtui
```

### 🐧 Debian / Ubuntu (APT)

我们提供了 APT 仓库，支持自动更新：

```bash
# 1. 添加 GPG 密钥
curl -fsSL https://ChanceFlow.github.io/OpenClashTUI/apt/KEY.gpg | sudo gpg --dearmor -o /usr/share/keyrings/clashtui.gpg

# 2. 添加软件源
echo "deb [signed-by=/usr/share/keyrings/clashtui.gpg] https://ChanceFlow.github.io/OpenClashTUI/apt stable main" | sudo tee /etc/apt/sources.list.d/clashtui.list

# 3. 更新并安装
sudo apt update
sudo apt install clashtui
```

### 📦 直接下载二进制

您可以直接从 [GitHub Releases](https://github.com/ChanceFlow/ClashTUI/releases) 页面下载适用于 Linux、macOS 和 Windows 的预编译二进制文件。

### 🛠️ 从源码编译

确保您已安装 [Rust 工具链](https://rustup.rs/)：

```bash
git clone https://github.com/ChanceFlow/ClashTUI.git
cd ClashTUI
cargo build --release
```

编译完成后，二进制文件位于 `target/release/clashtui`。您可以将其移动到 PATH 中的目录，例如 `/usr/local/bin`。

## 🚀 使用说明

### 启动 TUI

默认连接到 `127.0.0.1:9090`：

```bash
clashtui
```

指定 Clash 控制器地址和密钥（如果有）：

```bash
clashtui -c 192.168.1.5:9090 -s "your_secret_key"
```

### 命令行模式 (CLI)

不进入 TUI 界面，直接输出信息：

```bash
# 查看版本
clashtui version

# 列出所有代理组
clashtui groups

# 列出所有代理节点
clashtui proxies

# 列出所有规则
clashtui rules

# 刷新订阅（Proxy/Rule Providers）
clashtui refresh
```

## ⌨️ 快捷键

ClashTUI 提供了直观的快捷键系统，支持 Vim 风格导航。

### 🧭 导航

| 快捷键 | 功能 | 说明 |
| :--- | :--- | :--- |
| `j` / `↓` | 向下移动 | 选择列表中的下一项 |
| `k` / `↑` | 向上移动 | 选择列表中的上一项 |
| `h` / `←` | 左移 / 上一页 | 在代理界面切换到分组列表，或切换到上一个标签页 |
| `l` / `→` | 右移 / 下一页 | 在代理界面切换到节点列表，或切换到下一个标签页 |
| `Tab` | 切换焦点 | 在当前界面的不同区域间切换焦点 |
| `1` | 代理 (Proxies) | 切换到代理管理标签页 |
| `2` | 规则 (Rules) | 切换到规则列表标签页 |
| `3` | 连接 (Conns) | 切换到连接监控标签页 |

### ⚡ 操作

| 快捷键 | 功能 | 说明 |
| :--- | :--- | :--- |
| `Enter` | 选择 / 确认 | 选中当前代理节点 |
| `t` | 测试延迟 | 测试当前选中节点的延迟 (URL-Test) |
| `r` | 刷新 | 刷新当前列表数据 |
| `u` | 刷新订阅 | 刷新 Clash 的 Proxy/Rule Providers |
| `m` | 切换模式 | 在 Rule / Global / Direct 模式间循环切换 |
| `?` | 帮助 | 显示/隐藏快捷键帮助菜单 |
| `q` / `Esc` | 退出 | 退出程序或关闭当前弹窗 |

## ⚙️ Clash 配置要求

ClashTUI 依赖 Clash 的外部控制 API (External Controller)。请确保您的 `config.yaml` 中包含以下配置：

```yaml
external-controller: 127.0.0.1:9090
# external-ui: dashboard # 可选
secret: "" # 如果设置了密钥，启动 clashtui 时需通过 -s 参数指定
```

## 🤝 贡献

欢迎任何形式的贡献！如果您发现了 Bug 或有新功能建议：

1. 提交 [Issue](https://github.com/ChanceFlow/ClashTUI/issues) 反馈问题。
2. Fork 本仓库，创建您的特性分支 (`git checkout -b feature/AmazingFeature`)。
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)。
4. 推送到分支 (`git push origin feature/AmazingFeature`)。
5. 提交 Pull Request。

## 📄 许可证

本项目基于 [MIT 许可证](LICENSE) 开源。

---
<div align="center">
Made with ❤️ by ClashTUI Contributors
</div>
