# rspack-sources

Rust port of [`webpack-sources`](https://github.com/webpack/webpack-sources), providing composable sources and source map utilities for Rspack.

This crate now lives in the [Rspack monorepo](https://github.com/web-infra-dev/rspack/tree/main/crates/rspack_sources). Tests and CodSpeed benchmarks run through the Rspack workspace CI.

## Documentation

API reference: <https://docs.rs/rspack_sources>

## Source map text storage

`SourceMap` owns shared UTF-8 text using `bytes_str::BytesStr`. Its constructor and text setters
accept `BytesStr`; source/name getters return `&[BytesStr]`, and source contents use
`Option<BytesStr>` to distinguish missing content from an explicitly empty string.
Convert an owned `String` with `.into()`
to transfer its allocation, and use `BytesStr::from_str_slice` when an arbitrary borrowed string
must be copied. Fields live directly in `SourceMap`; cloning shares text and array storage,
and modifying a clone leaves other maps unchanged.

Maps no longer have a lifetime parameter or retain their generating Source graph. `OnSource`
and `OnName` callbacks receive shared `BytesStr` values. A custom Source should keep its original
text in shared storage and clone the handle when reporting sources or names. JSON construction
shares decoded fields in the input allocation, including escaped strings; an individual slice
can keep that entire allocation alive.

## SourceMap 文本存储

`SourceMap` 使用 `bytes_str::BytesStr` 拥有共享的 UTF-8 文本。构造器和文本 setter 接收
`BytesStr`，来源和名称的数组 getter 返回 `&[BytesStr]`，来源内容使用 `Option<BytesStr>`
区分缺失内容与显式空字符串。对拥有的 `String` 使用 `.into()` 接管 allocation；
需要复制任意借用字符串时使用 `BytesStr::from_str_slice`。字段直接保存在 `SourceMap` 中；
克隆 map 共享文本和数组存储，修改克隆不会改变原 map。

map 不再带生命周期参数，也不通过 Source 图保活文本。`OnSource` 和 `OnName` 回调接收
`BytesStr`；自定义 Source 应保存共享文本并在回调中克隆句柄。JSON 构造会共享输入中解码后的字段，
包括转义字符串；小切片仍可能保活完整输入 allocation。

## Credits

Thanks to the contributors of [`webpack-sources`](https://github.com/webpack/webpack-sources) for the original design.

## License

[MIT licensed](https://github.com/web-infra-dev/rspack/blob/main/crates/rspack_sources/LICENSE).
