# rspack_dojang

This is Rspack's fork of [dojang](https://github.com/kev0960/dojang). It was
migrated from [rstackjs/rspack-dojang](https://github.com/rstackjs/rspack-dojang)
at commit [`5b1b5e7c`](https://github.com/rstackjs/rspack-dojang/commit/5b1b5e7c8dcd3f78fc36cd24001bafa8722b7e1c)
so fixes and Rspack-specific features can be maintained with their consumers.

**Dojang** is an HTML template engine designed as a drop-in replacement for
[EJS](https://ejs.co/). It does not support all JavaScript syntax, but implements
the basic syntax used by Rspack.

## Features

- Supports basic JavaScript control flow (`if`, `for`, `while`, etc.).
- Supports script and output tags (`<%`, `<%-`, `<%=`).
- Supports calling external functions.

## How to use?

```rust
use rspack_dojang::Dojang;
use serde_json::Value;

// Create a template engine Dojang.
let mut dojang = Dojang::new();

// Load template file under '/my/template/files'
assert!(dojang.load("/my/template/files").is_ok());

// Render a template. "some_template" is one of the template files under /my/template/files.
// Note that the context should be provided as a serde_json value.
assert_eq!(
    dojang
        .render(
            "some_template",
            serde_json::from_str(r#"{ "a" : 1 }"#).unwrap()
        )
        .unwrap(),
    " Hi "
    );

assert_eq!(
    dojang
        .render(
            "some_template",
            serde_json::from_str(r#"{ "a" : 2 }"#).unwrap()
        )
        .unwrap(),
    "2"
    );
```
