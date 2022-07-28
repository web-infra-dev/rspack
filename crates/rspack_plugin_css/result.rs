#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::path::{Path, PathBuf};
use node_binding::{normalize_bundle_options, RawOptions};
use temp_test_utils::RawOptionsTestExt;
fn main() {
    let body = async {
        let mut cur_dir = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
        cur_dir = cur_dir.join("../../examples/bench");
        cur_dir = cur_dir.canonicalize().unwrap();
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&cur_dir)],
            ));
        };
        let options =
            normalize_bundle_options(RawOptions::from_fixture(&cur_dir)).unwrap_or_else(|_| {
                ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                    &["failed to normalize in fixtrue "],
                    &[::core::fmt::ArgumentV1::new_debug(&cur_dir)],
                ))
            });
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&options)],
            ));
        };
        let mut compiler = rspack::rspack(options, Default::default());
        let stats = compiler.run().await.unwrap_or_else(|_| {
            ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                &["failed to compile in fixtrue "],
                &[::core::fmt::ArgumentV1::new_debug(&cur_dir)],
            ))
        });
    };
    #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
