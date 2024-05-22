# 准备工作

Rspack 使用 [Rust](https://rust-lang.org/) 和 [NAPI-RS](https://napi.rs/) 构建，然后被发布为 [Node.js](https://nodejs.org/) 包.

## 安装 Rust

- 使用 [rustup](https://rustup.rs/) 安装 Rust.
- 如果你在使用 VS Code，我们推荐安装 [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) 扩展.

## 设置 Node.js

### 安装 Node.js

我们建议使用 Node.js 20 的 LTS 版本。

使用以下命令检查当前 Node.js 版本:

```bash
node -v
```

如果你当前环境中没有安装 Node.js，你可以使用 [nvm](https://github.com/nvm-sh/nvm) 或者 [fnm](https://github.com/Schniz/fnm) 来安装它。

这里有一个如何通过 nvm 安装的示例:

```bash
# Install Node.js LTS
nvm install 20 --lts

# Switch to Node.js LTS
nvm use 20
```
