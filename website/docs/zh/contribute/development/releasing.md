# Releasing

所有发布都是通过 GitHub Action 自动进行的。

所有 `@rspack/cli` 的发版可以在 [npm versions page](https://www.npmjs.com/package/@rspack/cli?activeTab=versions) 找到。它们被打上了 tag

- `latest` 和语义化版本 `x.y.z`
- `nightly`
- `canary`

## latest 的全量发布

[全量发布工作流](https://github.com/web-infra-dev/rspack/actions/workflows/release.yml?query=is%3Asuccess)
目前在每个周二被手动触发，配合全量发布的 release notes。

下面的 9 个目标产物会被构建

- x86_64-unknown-linux-gnu
- aarch64-unknown-linux-gnu
- x86_64-unknown-linux-musl
- aarch64-unknown-linux-musl
- i686-pc-windows-msvc
- x86_64-pc-windows-msvc
- aarch64-pc-windows-msvc
- x86_64-apple-darwin
- aarch64-apple-darwin

## Nightly

[nightly 发布工作流](https://github.com/web-infra-dev/rspack/actions/workflows/release-nightly.yml?query=is%3Asuccess)
在每天的 UTC 16:00:07 被触发，是 北京时间的凌晨 00:07 (偏移奇数分钟以避免 cron 作业同时触发)。

nightly 构建完全复制了全量发布构建，以便尽早发现错误。
