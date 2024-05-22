# Building

请查看 [准备工作](./prerequisites) 安装 Rust 和 Node.js 环境。

## 安装 Node.js 依赖

通过 [pnpm](https://pnpm.io/) 安装 Node.js 依赖。

```bash
# enable pnpm with corepack
corepack enable

# Install dependencies
pnpm i
```

## 构建 Rspack

- 执行 `cargo build` 编译 Rust 代码。
- 执行 `pnpm run build:cli:debug` 编译 Node.js 和 Rust 代码。

被编译的二进制产物位于 `packages/rspack-cli/bin/rspack` 。
