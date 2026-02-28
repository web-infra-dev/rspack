<picture>
  <img alt="Rspack Banner" src="https://assets.rspack.rs/rspack/rspack-banner.png">
</picture>

# @rspack/binding

Private node binding crate for rspack.

> Rspack's internal package, don't use it directly in your project, This package does _NOT_ follow [semantic versioning](https://semver.org/).

## Documentation

See [https://rspack.rs](https://rspack.rs) for details.

## Update Wasm binding

The generation of `rspack.wasi-browser.js` and `rspack.wasi.js` is disabled by default because `@napi/cli` produces unstable output for these files. To update the Wasm bindings, add `wasm32-wasip1-threads` to the `napi.targets` field in `package.json` before building the project.

## License

Rspack is [MIT licensed](https://github.com/web-infra-dev/rspack/blob/main/LICENSE).
