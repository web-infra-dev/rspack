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
Immutable field arrays use `ThinArc<(), T>` to keep their elements, length, and reference count
in one allocation. Freezing a builder's `Vec` moves element handles without copying UTF-8 text.

Maps no longer have a lifetime parameter or retain their generating Source graph. `OnSource`
and `OnName` callbacks borrow `BytesStr` values for the duration of each call. A custom Source
can report references to its shared text directly. Consumers clone a handle only when retaining
it in a deduplication table, resulting map, or cache; callbacks may also receive temporary joined
source paths, so their references must not escape the call. JSON construction
shares decoded fields in the input allocation, including escaped strings; an individual slice
can keep that entire allocation alive.
Passing an empty vector to `set_sources_content` copies the remaining metadata into independent
storage so this map no longer retains the removed source contents through shared JSON slices.

## SourceMap 文本存储

`SourceMap` 使用 `bytes_str::BytesStr` 拥有共享的 UTF-8 文本。构造器和文本 setter 接收
`BytesStr`，来源和名称的数组 getter 返回 `&[BytesStr]`，来源内容使用 `Option<BytesStr>`
区分缺失内容与显式空字符串。对拥有的 `String` 使用 `.into()` 接管 allocation；
需要复制任意借用字符串时使用 `BytesStr::from_str_slice`。字段直接保存在 `SourceMap` 中；
克隆 map 共享文本和数组存储，修改克隆不会改变原 map。
不可变字段数组使用 `ThinArc<(), T>`，把元素、长度和引用计数放在一次分配中。
冻结构建过程的 `Vec` 时会搬移元素句柄，不复制 UTF-8 文本。

map 不再带生命周期参数，也不通过 Source 图保活文本。`OnSource` 和 `OnName` 回调接收
`&BytesStr`，借用只在当前回调期间有效；自定义 Source 可以直接传递共享文本的引用。
只有保存到去重表、结果 map 或缓存时才克隆句柄。拼接后的来源路径可能是临时值，
回调中的引用不能逃逸。JSON 构造会共享输入中解码后的字段，
包括转义字符串；小切片仍可能保活完整输入 allocation。
向 `set_sources_content` 传入空数组时，会将其余元数据复制到独立存储，
避免当前 map 通过共享 JSON 切片继续保留已移除的源码内容。

## Credits

Thanks to the contributors of [`webpack-sources`](https://github.com/webpack/webpack-sources) for the original design.

## License

[MIT licensed](https://github.com/web-infra-dev/rspack/blob/main/crates/rspack_sources/LICENSE).
