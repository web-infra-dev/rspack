# css-hmr-perf

Measures the browser-side cost of HMR updates in an app with a large
extracted stylesheet, comparing the published rspack (`rspack-npm`, old
runtime: reload the stylesheet of every updated chunk) against the workspace
build (manifest-driven precise css updates).

Latency is measured from the first `hot-update` request seen by the browser,
so server-side rebuild time (dev-profile vs release binding) stays out of the
comparison.

## Run

```bash
pnpm install          # repo root
node bench.mjs --old  # published @rspack/core → results-old.json
node bench.mjs --new  # workspace @rspack/core → results-new.json
node report.mjs       # merge into report.html
```

Scenarios per size (generated stylesheets from 0.5MB to 5MB in 0.5MB steps):

- **js-only edit** — only `marker.js` changes; the stylesheet is untouched.
  The old runtime still refetches and re-applies it, the new runtime leaves
  it alone (`css requests` should be 0).
- **css edit** — one rule's color changes; both runtimes must apply it.
