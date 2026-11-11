# rspack-sources

Rust port of [`webpack-sources`](https://github.com/webpack/webpack-sources), providing composable sources and source map utilities for Rspack.

This crate now lives in the [Rspack monorepo](https://github.com/web-infra-dev/rspack/tree/main/crates/rspack_sources). Tests and CodSpeed benchmarks run through the Rspack workspace CI.

## Documentation

API reference: <https://docs.rs/rspack_sources>

## Incremental mapping encoding

Use `FullMappingsEncoder` when mappings arrive through callbacks. Call
`encode(&mapping)` in generated-position order, then call `finish()` to consume
the encoder and obtain the compacted VLQ string. Only the encoded string is
retained. Use `encode_mappings(iterator)` when the mappings already form an iterator.

See [中文说明](README.zh-CN.md) for the Chinese guide.

## Credits

Thanks to the contributors of [`webpack-sources`](https://github.com/webpack/webpack-sources) for the original design.

## License

[MIT licensed](https://github.com/web-infra-dev/rspack/blob/main/crates/rspack_sources/LICENSE).
