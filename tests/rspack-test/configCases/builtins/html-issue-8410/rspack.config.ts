import { defineConfig } from '@rspack/cli';
import { type Compiler, rspack } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('Plugin', (compilation) => {
      const hooks = rspack.HtmlRspackPlugin.getCompilationHooks(compilation);
      hooks.alterAssetTags.tapPromise('Plugin', async (data) => {
        for (const tag of data.assetTags.scripts) {
          if (tag.tagName === 'script') {
            tag.attributes.defer = true;
          }
        }
        return data;
      });
    });
  }
}

export default defineConfig({
  entry: {
    foorBar: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [new rspack.HtmlRspackPlugin({}), new Plugin()],
});
