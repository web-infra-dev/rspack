import { defineConfig } from '../../../dist/index.js';
import type { Compiler } from '@rspack/core';

const PLUGIN_NAME = 'MyPlugin';

class MyPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, () => {
      console.log('MyPlugin: The Rspack build process is starting!');
    });
  }
}

export default defineConfig({
  mode: 'development',
  entry: './entry.js',
  extends: './base.config.mts',
  // Override or add to the base configuration
  output: {
    filename: '[name].bundle.js',
  },
  plugins: [
    // "...", // uncomment this to inherit plugins, but it's also a type error
    new MyPlugin(),
  ],
});
