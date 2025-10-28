# Releasing

Rspack releases are automated through GitHub Actions.

You can view all released versions on the npm version pages of [@rspack/core](https://www.npmjs.com/package/@rspack/core?activeTab=versions) and [@rspack/cli](https://www.npmjs.com/package/@rspack/cli?activeTab=versions).

## Latest release

The latest stable release follows the Semantic Versioning specification (x.y.z).

The [full release workflow](https://github.com/web-infra-dev/rspack/actions/workflows/release.yml?query=is%3Asuccess) is triggered manually by Rspack maintainers on Tuesday with the complete release notes.

During the release, the following binary artifacts for the target platforms are built:

- x86_64-unknown-linux-gnu
- aarch64-unknown-linux-gnu
- x86_64-unknown-linux-musl
- aarch64-unknown-linux-musl
- i686-pc-windows-msvc
- x86_64-pc-windows-msvc
- aarch64-pc-windows-msvc
- x86_64-apple-darwin
- aarch64-apple-darwin

### Release steps

1. Create a new branch, for example `release/v1.0.0`.
2. Update the version using the `pnpm x version` command on the branch.

```bash
# Release a patch version
pnpm x version patch

# Release a minor version
pnpm x version minor

# Release a major version
pnpm x version major

# Release an alpha version
pnpm x version patch --pre alpha

# Release a beta version
pnpm x version patch --pre beta

# Release a rc version
pnpm x version patch --pre rc
```

3. Commit the code and push to the remote branch.

```bash
git add .
git commit -m "chore: release v1.0.0"
git push origin release/vx.y.z
```

4. Create a PR with the title `chore: release v1.0.0`.
5. Run the [Ecosystem CI workflow](https://github.com/web-infra-dev/rspack/actions/workflows/ecosystem-ci.yml) to ensure all ecosystem projects are working properly.
6. Run the full release workflow on the release branch:
   - [Release Full](https://github.com/web-infra-dev/rspack/actions/workflows/release.yml): Publish npm packages to registry
   - [Release Crates](https://github.com/web-infra-dev/rspack/actions/workflows/release-crates.yml): Publish Rust crates to crates.io
7. After the release, merge the PR to the `main` branch.
8. Generate the [GitHub release note](https://github.com/web-infra-dev/rspack/releases), and add highlights information.

## Canary release

Canary is the pre-release version for testing and verifying new features.

Releasing a canary version does not require manually creating a branch or updating the version, it only requires Rspack maintainers to trigger the [Canary release workflow](https://github.com/web-infra-dev/rspack/actions/workflows/release-canary.yml).
