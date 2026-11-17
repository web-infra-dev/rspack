---
description: 'Rspack 贡献指南中的准备工作章节，涵盖安装 Rust和安装 Node.js等核心内容，并说明本地准备要求。'
---

# 准备工作

Rspack 使用 [Rust](https://rust-lang.org/) 和 [NAPI-RS](https://napi.rs/) 构建，然后被发布为 [Node.js](https://nodejs.org/) 包。

## 安装 Rust

- 使用 [rustup](https://rustup.rs/) 安装 Rust。
- 如果你在使用 VS Code，我们推荐安装 [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) 扩展。

在 macOS 上构建前，请在仓库根目录安装指定版本的 LLD 链接器：

```bash
./scripts/macos-linker.sh --setup
```

该命令会安装一个独立的 Rust 工具链，以使用其中支持新版 Xcode SDK 的链接器。Rspack 的编译器仍使用 `rust-toolchain.toml` 中指定的版本。

## 安装 Node.js

使用最新的 Node.js LTS 版本。安装说明请参考 [安装 Node.js](https://nodejs.org/zh-cn/download)。
