# rspack-sources

`webpack-sources` 的 Rust 实现，为 Rspack 提供可组合的源代码和 source map 工具。

本 crate 位于 [Rspack 主仓库](https://github.com/web-infra-dev/rspack/tree/main/crates/rspack_sources)，测试和 CodSpeed 基准由工作区 CI 运行。

## 文档

API 参考：<https://docs.rs/rspack_sources>。

## 增量编码映射

当映射通过回调逐条产生时，使用 `FullMappingsEncoder`。
按生成位置的顺序调用 `encode(&mapping)`，最后调用 `finish()` 消费编码器，取得压缩后的 VLQ 字符串。
编码器只保留编码结果，不保存已处理的映射。
如果映射已经是迭代器，使用 `encode_mappings(iterator)`。

## 致谢

感谢 [webpack-sources](https://github.com/webpack/webpack-sources) 的贡献者提供原始设计。

## 许可证

[MIT 许可证](https://github.com/web-infra-dev/rspack/blob/main/crates/rspack_sources/LICENSE)。
