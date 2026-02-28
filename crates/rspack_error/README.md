# rspack_error

## Testing

### Prerequisite

- Install `cargo-insta`, more details reference https://crates.io/crates/cargo-insta

### How

1. Run `cargo test`.
2. If any updates related to diagnostic emitting, the testing maybe fail.
3. Checking if the `fixtures.new.snap` is expected, if true then run `cargo insta accept`,
   more document you could see https://github.com/mitsuhiko/insta.
   Else adjust you program logic then goto `1`
