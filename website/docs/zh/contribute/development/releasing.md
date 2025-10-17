# Releasing

Rspack 的版本发布通过 GitHub Actions 自动完成。

你可以在 [@rspack/core](https://www.npmjs.com/package/@rspack/core?activeTab=versions) 和 [@rspack/cli](https://www.npmjs.com/package/@rspack/cli?activeTab=versions) 的 npm 版本页面查看所有已发布的版本。

## Latest 发布

Latest 是最新的稳定版本，遵循 Semantic Versioning 语义化版本号规范（`x.y.z`）。

[全量发布工作流](https://github.com/web-infra-dev/rspack/actions/workflows/release.yml?query=is%3Asuccess) 会在每周二由 Rspack 维护者手动触发，并带有完整的 release notes。

在发布过程中，会构建以下目标平台的二进制产物：

- x86_64-unknown-linux-gnu
- aarch64-unknown-linux-gnu
- x86_64-unknown-linux-musl
- aarch64-unknown-linux-musl
- i686-pc-windows-msvc
- x86_64-pc-windows-msvc
- aarch64-pc-windows-msvc
- x86_64-apple-darwin
- aarch64-apple-darwin

### 发布步骤

1. 创建一个新分支，例如 `release/v1.0.0`。
2. 在分支上使用 `pnpm x version` 命令更新版本号。

```bash
# 发布 patch 版本
pnpm x version patch

# 发布 minor 版本
pnpm x version minor

# 发布 major 版本
pnpm x version major

# 发布 alpha 版本
pnpm x version patch --pre alpha

# 发布 beta 版本
pnpm x version patch --pre beta

# 发布 rc 版本
pnpm x version patch --pre rc
```

3. 提交代码并推送到远程分支。

```bash
git add .
git commit -m "chore: release v1.0.0"
git push origin release/vx.y.z
```

4. 创建一个 PR，标题为 `chore: release v1.0.0`。
5. 执行 [Ecosystem CI 工作流](https://github.com/web-infra-dev/rspack/actions/workflows/ecosystem-ci.yml)，确保所有生态项目都能正常工作。
6. 在 release 分支上执行全量发布工作流：
   - [Release Full](https://github.com/web-infra-dev/rspack/actions/workflows/release.yml): 发布 npm 包到 registry
   - [Release Crates](https://github.com/web-infra-dev/rspack/actions/workflows/release-crates.yml): 发布 Rust crates 到 crates.io
7. 发布完成后，合并 PR 到 `main` 分支。
8. 生成 GitHub [release note](https://github.com/web-infra-dev/rspack/releases)，补充 highlights 信息。

## Canary 发布

Canary 是 Rspack 的预发布版本，用于测试和验证新功能。

发布 canary 版本不需要手动创建分支或更新版本号，只需要由 Rspack 维护者执行 [Canary 发布工作流](https://github.com/web-infra-dev/rspack/actions/workflows/release-canary.yml) 即可。
