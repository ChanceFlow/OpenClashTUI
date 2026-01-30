# ClashTUI

一个用 Rust 开发的 Clash 外部控制端口 TUI 工具，用于管理代理和规则。

## 功能

- 📋 查看和切换代理组中的代理节点
- 📜 查看所有路由规则
- ⚡ 测试代理节点延迟
- 🎨 美观的终端界面 (TUI)
- 💻 支持命令行模式快速操作

## 安装

```bash
cargo build --release
```

编译后的二进制文件位于 `target/release/clashtui`。

## 使用方法

### TUI 模式（默认）

```bash
# 使用默认地址 127.0.0.1:9090
clashtui

# 指定控制器地址
clashtui -c 127.0.0.1:9090

# 如果配置了 secret
clashtui -c 127.0.0.1:9090 -s your_secret
```

### 命令行模式

```bash
# 查看 Clash 版本
clashtui version

# 列出所有代理组
clashtui groups

# 列出所有代理
clashtui proxies

# 列出所有规则
clashtui rules
```

## 快捷键

### 导航

| 快捷键 | 功能 |
|--------|------|
| `↑` / `k` | 向上移动 |
| `↓` / `j` | 向下移动 |
| `Tab` | 在组列表和代理列表间切换焦点 |
| `1` | 切换到代理标签页 |
| `2` | 切换到规则标签页 |

### 操作

| 快捷键 | 功能 |
|--------|------|
| `Enter` | 选择当前代理 |
| `t` | 测试当前代理延迟 |
| `r` | 刷新数据 |

### 通用

| 快捷键 | 功能 |
|--------|------|
| `?` | 显示/隐藏帮助 |
| `q` / `Esc` | 退出 |

## Clash 配置

确保你的 Clash 配置文件中启用了外部控制：

```yaml
external-controller: 127.0.0.1:9090
secret: ""  # 可选，设置 API 密钥
```

## API 参考

本工具基于 [Clash RESTful API](https://tangwenlongno1.github.io/clash-verge-rev.github.io/api/index.html) 实现，支持以下端点：

- `GET /proxies` - 获取代理信息
- `PUT /proxies/:name` - 选择代理
- `GET /proxies/:name/delay` - 测试代理延迟
- `GET /rules` - 获取规则信息
- `GET /version` - 获取版本信息

## 依赖

- `reqwest` - HTTP 客户端
- `ratatui` - TUI 框架
- `crossterm` - 终端控制
- `clap` - 命令行参数解析
- `serde` / `serde_json` - JSON 序列化
- `anyhow` / `thiserror` - 错误处理

## License

MIT
