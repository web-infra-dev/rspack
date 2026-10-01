import { defineConfig } from '@rspack/cli';
import type { Module, Compiler } from '@rspack/core';

let firstNameRan = false;
const events: string[] = [];
const layerArgumentCounts = new Set<number>();
const isSharedModule = (module: Module) =>
  /[\\/]shared\.js$/.test(module.nameForCondition() || '');

class AssertCallbackOrderPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.done.tap('AssertCallbackOrderPlugin', (stats) => {
      expect(events).toEqual(['first:name', 'second:test:true']);
      expect([...layerArgumentCounts]).toEqual([1]);

      const assets = stats.toJson({ all: false, assets: true }).assets;
      expect(assets?.some((asset) => asset.name === 'second.js')).toBe(true);
      expect(assets?.some((asset) => asset.name === 'first.js')).toBe(false);
    });
  }
}

export default defineConfig({
  mode: 'production',
  target: 'node',
  entry: './index.js',
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  optimization: {
    minimize: false,
    concatenateModules: false,
    chunkIds: 'named',
    moduleIds: 'named',
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        first: {
          test: /shared\.js$/,
          layer() {
            layerArgumentCounts.add(arguments.length);
            return true;
          },
          minChunks: 2,
          enforce: true,
          priority: 0,
          chunks(chunk) {
            return chunk.name !== 'c';
          },
          name(module) {
            if (isSharedModule(module)) {
              events.push('first:name');
              firstNameRan = true;
            }
            return 'first';
          },
        },
        second: {
          test(module) {
            if (!isSharedModule(module)) {
              return false;
            }
            events.push(`second:test:${firstNameRan}`);
            return firstNameRan;
          },
          minChunks: 2,
          enforce: true,
          priority: 0,
          name: 'second',
        },
      },
    },
  },
  plugins: [new AssertCallbackOrderPlugin()],
});
