import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  optimization: {
    concatenateModules: false,
    inlineExports: false,
    mangleExports: false,
    // Explicitly minify: value bindings must be emitted as direct
    // `Object.defineProperty(..., { value })` calls so they survive SWC
    // minify. SWC is free to rewrite the 3-arg `__webpack_require__.d`
    // runtime (e.g. to per-key `(exports, name, getter)` form), which would
    // silently drop a third `values` argument.
    minimize: true,
  },
});
