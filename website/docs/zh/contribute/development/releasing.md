# Releasing

所有发布都是通过 GitHub Action 自动进行的。

所有 `@rspack/cli` 的发版可以在 [npm versions page](https://www.npmjs.com/package/@rspack/cli?activeTab=versions) 找到。它们被打上了 tag

- `latest` 和语义化版本 `x.y.z`
- `nightly`
- `canary`

## Latest 版本全量发布

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

## Canary

[canary 发布工作流](https://github.com/web-infra-dev/rspack/actions/workflows/release-canary.yml) 需要手动触发。

## nightly/canary 版本的使用

Rspack 的 nightly/canary 版本的包名会在增加 `-canary` 后缀之后发布，需要配合包管理器 npm/yarn/pnpm 的 overrides 功能使用

以 pnpm 为例:

```json title=package.json
{
  "pnpm": {
    "overrides": {
      "@rspack/binding": "npm:@rspack/binding-canary@nightly",
      "@rspack/core": "npm:@rspack/core-canary@nightly",
      "@rspack/plugin-react-refresh": "npm:@rspack/plugin-react-refresh@nightly"
    },
    "peerDependencyRules": {
      "allowAny": ["@rspack/*"]
    }
  }
}
```

Rspack 社区提供了 [install-rspack](https://github.com/rspack-contrib/install-rspack) 工具来快速修改 Rspack 版本:

```shell
npx install-rspack --version nightly # Get latest nightly npm tag version
npx install-rspack --version 0.7.5-canary-d614005-20240625082730 # A specific canary version
```
