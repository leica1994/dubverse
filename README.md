# Dubverse

视频转录、翻译、TTS 配音的桌面工具。基于 Tauri 2 + Vue 3 + TypeScript 构建。

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue 3, TypeScript 5.6, Vite 6 |
| 后端 | Rust, Tauri 2 |
| 包管理 | npm |

## 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.70
- Windows: [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

## 快速开始

```bash
# 安装前端依赖
npm install

# 启动开发环境（自动启动 Vite + Tauri 窗口）
npm run tauri dev
```

首次启动会编译 Rust 依赖，耗时较长，后续启动为增量编译。

## 构建发布

```bash
npm run tauri build
```

产物位于 `src-tauri/target/release/bundle/`。

## 项目结构

```
src/                  # Vue 3 前端源码
src-tauri/            # Rust 后端源码
  src/lib.rs          # Tauri 命令处理
  tauri.conf.json     # 应用配置
```

## IDE 推荐

- [VS Code](https://code.visualstudio.com/)
  - [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
  - [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
  - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
