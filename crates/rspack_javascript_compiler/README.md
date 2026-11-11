# Rspack JavaScript compiler

## Source-map writer

Mapped output uses a private `WriteJs` implementation adapted from SWC 33.0.0.
It keeps source positions only for the current generated position, rather than
a hash set of every emitted mapping. Generated positions advance monotonically,
so mappings at earlier positions cannot be duplicates of a new mapping.
Each accepted mapping goes directly to Rspack's VLQ encoder. Neither raw SWC
mappings nor converted mappings accumulate in a temporary vector.

The writer preserves SWC's pending mappings, indentation, UTF-16 columns,
CRLF handling, and literal boundaries. Rspack still uses SWC's
`omit_trailing_semi` adapter. Output without source maps uses SWC's original writer.

Differential tests in `rspack_javascript_compiler_testing` compare output bytes
and raw mappings against the pinned SWC writer. Run them when upgrading SWC:

```sh
cargo test -p rspack_javascript_compiler_testing
```

## License

Rspack's code is MIT licensed. The adapted source-map writer is Apache-2.0
licensed; see [LICENSE-APACHE](LICENSE-APACHE) and its source header.
