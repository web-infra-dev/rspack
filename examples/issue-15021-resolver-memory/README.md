# Rspack issue #15021 resolver memory reproduction

This standalone Rust program demonstrates the resolver-side memory multiplier
behind [web-infra-dev/rspack#15021](https://github.com/web-infra-dev/rspack/issues/15021).

It builds a synthetic TypeScript project-reference graph, repeatedly resolves
the same request, and retains every `ResolveContext`. Retaining the contexts
models Rspack keeping file and missing dependencies for incremental
invalidation during `build_module_graph`.

The comparison is intentionally between:

- `rspack_resolver` 0.8.0, used by `@rspack/core` 2.0.1 through 2.0.4.
- `rspack_resolver` 0.9.1, first used by `@rspack/core` 2.0.5.

Resolver 0.9 tracks the root tsconfig, extended configs, and project-reference
configs as file dependencies. With `references: auto`, that dependency closure
is copied into every resolve result.

## Run

Run the two versions in separate processes so their peak RSS values are
independent:

```bash
cargo run --release --locked -- 0.8 300 1000
cargo run --release --locked -- 0.9 300 1000
```

Arguments are `<resolver-version> <project-reference-count> <resolve-count>`.
The program supports macOS and Linux.

Example output from macOS:

```text
resolver=0.8 contexts=1000 entries_per_context=9 total_retained_entries=9000 peak_rss_mib=5.17
resolver=0.9 contexts=1000 entries_per_context=311 total_retained_entries=311000 peak_rss_mib=56.41
```

With no project references, the fixed overhead is comparable:

```bash
cargo run --release --locked -- 0.8 0 1000
cargo run --release --locked -- 0.9 0 1000
```

This reproduction uses no symlinks. Symlink-heavy layouts can increase the
number and length of resolver dependency paths, but they are not required to
trigger the project-reference multiplier.

## Relevant changes

- [Track tsconfig files as dependencies](https://github.com/rstackjs/rspack-resolver/pull/200)
- [Support transitive project references](https://github.com/rstackjs/rspack-resolver/pull/213)
- [Carry dependency paths as `ResolverPath`](https://github.com/rstackjs/rspack-resolver/pull/232)
- [Rspack integration](https://github.com/web-infra-dev/rspack/pull/14120)
