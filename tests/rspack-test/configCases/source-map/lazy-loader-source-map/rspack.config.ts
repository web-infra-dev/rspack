import { defineConfig, definePlugin } from '@rspack/cli';
import binding from '../../../../../crates/node_binding/binding.js';

const { JsSourceMap } = binding;

export default defineConfig({
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /fixture\.js$/,
        use: [
          './verify.mjs',
          'builtin:test-passthrough-loader',
          './exercise.mjs',
          { loader: 'builtin:swc-loader', options: {} },
        ],
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const originalToJson = JsSourceMap.prototype.toJson;
        const originalStringify = JSON.stringify;
        const state = {
          serializations: 0,
          protectedMaps: new WeakSet<object>(),
          retained: [] as any[],
          exercised: 0,
        };
        (compiler as any).__sourceMapTest = state;
        const restore = () => {
          JsSourceMap.prototype.toJson = originalToJson;
          JSON.stringify = originalStringify;
        };
        compiler.hooks.beforeCompile.tap('LazyLoaderSourceMap', () => {
          state.serializations = 0;
          state.exercised = 0;
          state.retained = [];
          JsSourceMap.prototype.toJson = function () {
            const json = originalToJson.call(this);
            if (json.includes('lazy-loader-source-map')) state.serializations++;
            return json;
          };
          JSON.stringify = ((value, ...args) => {
            if (value && state.protectedMaps.has(value)) {
              throw new Error(
                'An unmodified loader map must not be stringified',
              );
            }
            return Reflect.apply(originalStringify, JSON, [value, ...args]);
          }) as typeof JSON.stringify;
        });
        compiler.hooks.afterCompile.tap('LazyLoaderSourceMap', () => {
          try {
            if (state.exercised > 0) {
              // Nine maps are read in exercise, and ten survive the native loader
              // and are read in verify. Passing or dropping a map needs no JSON.
              expect(state.exercised).toBe(11);
              expect(state.serializations).toBe(19);
              for (const map of state.retained) {
                // An unread input remains usable after it has been returned to Rust.
                expect(map.version).toBe(3);
                expect(Array.isArray(map.sources)).toBe(true);
              }
            }
          } finally {
            restore();
          }
        });
        compiler.hooks.failed.tap('LazyLoaderSourceMap', restore);
      },
    }),
  ],
});
