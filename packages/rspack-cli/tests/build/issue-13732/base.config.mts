import { defineConfig } from '../../../dist/index.js';
import type { Compiler } from '@rspack/core';

const PLUGIN_NAME = 'MyBasePlugin';

export class MyBasePlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, () => {
      console.log('MyBasePlugin: The Rspack build process is starting!');
    });
  }
}

export default defineConfig({
  plugins: [new MyBasePlugin()],
});
