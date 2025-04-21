# rspack_tracing_chrome

This is a fork of [tracing-chrome](https://github.com/thoren-d/tracing-chrome) to add better support for perfetto to Rspack like:

* support merge Rust & JS tracing
* support use `id2` to different span in same track for perfetto, related to https://github.com/thoren-d/tracing-chrome/issues/24
* ...