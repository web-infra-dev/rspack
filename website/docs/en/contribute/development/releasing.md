# Releasing

All releases are automated through GitHub actions.

All published releases of `@rspack/cli` can be found on the [npm versions page](https://www.npmjs.com/package/@rspack/cli?activeTab=versions). They are tagged as

- `latest` with semver version `x.y.z`
- `nightly`
- `canary`

## Latest Full Release

The [full release workflow](https://github.com/web-infra-dev/rspack/actions/workflows/release.yml?query=is%3Asuccess)
is currently triggered manually every Tuesday with full release notes.

The following 9 targets are built

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

The [nightly release workflow](https://github.com/web-infra-dev/rspack/actions/workflows/release-nightly.yml?query=is%3Asuccess)
is triggered every day at UTC 16:00:07, which is 00:07 AM Beijing Time (offset with an odd minute to avoid cron jobs firing off at the same time).

The nightly build fully replicates the full release build for catching errors early.
