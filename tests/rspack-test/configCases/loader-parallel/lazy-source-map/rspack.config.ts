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
          {
            loader: './exercise.mjs',
            options: {},
            parallel: { maxWorkers: 2 },
          },
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
        const protectedMaps = new WeakSet<object>();
        let serializations = 0;
        (compiler as any).__sourceMapTest = { protectedMaps };
        const restore = () => {
          JsSourceMap.prototype.toJson = originalToJson;
          JSON.stringify = originalStringify;
        };
        compiler.hooks.beforeCompile.tap('ParallelLazySourceMap', () => {
          serializations = 0;
          JsSourceMap.prototype.toJson = function () {
            const json = originalToJson.call(this);
            if (json.includes('lazy-source-map')) serializations++;
            return json;
          };
          JSON.stringify = ((value, ...args) => {
            if (value && protectedMaps.has(value)) {
              throw new Error(
                'An unmodified loader map must not be stringified',
              );
            }
            return Reflect.apply(originalStringify, JSON, [value, ...args]);
          }) as typeof JSON.stringify;
        });
        compiler.hooks.afterCompile.tap('ParallelLazySourceMap', () => {
          try {
            // Nine worker inputs are read and ten output maps are read on the
            // main thread. Unread and discarded maps never cross the worker boundary.
            if (serializations) expect(serializations).toBe(19);
          } finally {
            restore();
          }
        });
        compiler.hooks.failed.tap('ParallelLazySourceMap', restore);
      },
    }),
  ],
});
