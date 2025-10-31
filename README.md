<p align="center">
    <img width="150" src="./public/logo.png" alt="NetworkTools Logo">
</p>
<h1 align="center">NetworkTools</h1>
<p align="center">
一个功能强大的网络工具集，基于 <a href="https://nuxt.com">Nuxt 4</a> 和 <a href="https://v2.tauri.app">Tauri 2</a> 构建
<br>
专业的网络分析与管理工具！
</p>

<br />

<p float="left">
	<img src="https://img.shields.io/badge/version-1.5.0-blue" />
	<img src="https://img.shields.io/badge/rust-1.70+-orange" />
	<img src="https://img.shields.io/badge/nuxt-4.0-green" />
	<img src="https://img.shields.io/badge/license-MIT-purple" />
</p>

<br />

## 🌟 主要功能

### 📡 IPv4 工具
- **子网计算器** - 快速计算IPv4子网信息
- **地址汇总** - 批量汇总和优化IPv4地址段
- **公网IP查询** - 获取当前公网IP地址

### 🌐 IPv6 工具
- **IPv6子网计算** - 支持IPv6地址段分析
- **地址格式验证** - IPv6地址格式检查

### 🌍 IP地理位置
- **IP地理位置查询** - 精确的IP地址地理位置信息
- **数据库信息** - 支持GeoIP数据库查询
- **调试工具** - IP查询调试功能

### 🔧 NAT工具
- **NAT配置解析** - 网络地址转换配置分析
- **规则验证** - NAT规则语法检查

## 🛠️ 技术栈

- **前端框架**: Nuxt v4 + NuxtUI v4
- **样式框架**: TailwindCSS v4
- **桌面应用**: Tauri v2
- **编程语言**: TypeScript + Rust
- **代码质量**: ESLint + Prettier
- **自动化**: Tauri函数自动导入

## 📋 系统要求

### 运行环境
- **操作系统**: Windows 10+, macOS 10.15+, Linux (Ubuntu 18.04+)
- **Rust环境**: 1.70+ (参考 [Tauri prerequisites](https://tauri.app/start/prerequisites))
- **包管理器**: bun (强制要求，如需更换请更新 `package.json` 和 `tauri.conf.json`)

### 开发环境
- **Node.js**: 18+
- **内存**: 至少 4GB RAM
- **存储**: 2GB 可用空间

## 🚀 快速开始

### 1. 环境准备
```bash
# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 bun
curl -fsSL https://bun.sh/install | bash
```

### 2. 项目设置
```bash
# 克隆项目
git clone https://github.com/mam15mon/networktools.git
cd networktools

# 安装依赖
bun install

# 启动开发服务器
bun run tauri:dev
```

这将启动 Nuxt 前端（端口 3000）并打开 Tauri 桌面应用。

## 📦 构建部署

### 生产构建
```bash
# 构建优化版本
bun run tauri:build
```

### 调试构建
```bash
# 构建调试版本（包含开发者工具）
bun run tauri:build:debug
```

构建产物将生成在 `src-tauri/target` 目录中。

## 🔧 配置说明

### 端口配置
- **前端端口**: 3000 (可在 `nuxt.config.ts` 中修改)
- **Tauri端口**: 3001 (可在 `tauri.conf.json` 中修改)

### 权限配置
Tauri v2 需要在 `src-tauri/capabilities/main.json` 中配置权限：
- 文件系统访问
- 网络请求
- 系统通知
- 操作系统信息

### Tauri插件
项目集成了以下 Tauri 插件：
- `tauri-plugin-shell` - Shell命令执行
- `tauri-plugin-notification` - 系统通知
- `tauri-plugin-os` - 操作系统信息
- `tauri-plugin-fs` - 文件系统操作
- `tauri-plugin-store` - 本地数据存储
- `tauri-plugin-dialog` - 对话框

## 📁 项目结构

```
networktools/
├── app/                    # Nuxt 前端代码
│   ├── components/        # Vue 组件
│   ├── layouts/           # 布局文件
│   ├── pages/             # 页面路由
│   └── app.vue           # 根组件
├── src-tauri/             # Tauri 后端代码
│   ├── src/              # Rust 源码
│   │   ├── ipv4_subnet.rs
│   │   ├── ipv6_subnet.rs
│   │   ├── ipv4_summary.rs
│   │   ├── ip_location.rs
│   │   ├── nat_parser.rs
│   │   └── lib.rs
│   ├── Cargo.toml        # Rust 依赖配置
│   └── tauri.conf.json   # Tauri 应用配置
├── package.json          # Node.js 依赖配置
├── nuxt.config.ts        # Nuxt 框架配置
└── README.md            # 项目文档
```

## 🧪 开发指南

### 添加新的网络工具
1. 在 `src-tauri/src/` 中创建新的 Rust 模块
2. 在 `lib.rs` 中注册 Tauri 命令
3. 在前端 `app/` 中创建对应的 UI 组件
4. 更新 `app/modules/tauri.ts` 以支持新的 Tauri 函数自动导入

### 代码规范
- 遵循 Rust 官方代码风格
- TypeScript 严格模式
- ESLint + Prettier 自动格式化
- 提交前必须通过 linting 检查

## 🐛 故障排除

### 常见问题

**Q: 构建失败提示 Rust 版本过低**
A: 请运行 `rustup update` 更新 Rust 到最新版本

**Q: 前端热重载不工作**
A: 检查端口 3000 是否被占用，或修改 `nuxt.config.ts` 中的端口配置

**Q: Tauri 窗口无法打开**
A: 确保系统已安装必要的图形库依赖，参考 Tauri 官方文档

**Q: IP地理位置查询不准确**
A: 需要更新 GeoIP 数据库，或检查网络连接

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支: `git checkout -b feature/amazing-feature`
3. 提交更改: `git commit -m 'Add amazing feature'`
4. 推送分支: `git push origin feature/amazing-feature`
5. 提交 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## 🙏 致谢

- [Nuxt](https://nuxt.com) - 强大的 Vue.js 框架
- [Tauri](https://tauri.app) - 轻量级桌面应用框架
- [NuxtUI](https://ui.nuxt.com) - 优雅的 UI 组件库
- [TailwindCSS](https://tailwindcss.com) - 实用优先的 CSS 框架


---

<p align="center">
  Made with ❤️ by mam15mon
</p>