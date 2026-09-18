import { rspack } from '@rspack/core';

class Plugin {
  apply(compiler) {
    compiler.hooks.compilation.tap('Plugin', (compilation) => {
      const hooks = rspack.HtmlRspackPlugin.getCompilationHooks(compilation);
      hooks.alterAssetTags.tapPromise('Plugin', async (data) => {
        for (const tag of data.assetTags.scripts) {
          if (tag.tagName === 'script') {
            tag.attributes.defer = true;
          }
        }
      });
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    foorBar: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [new rspack.HtmlRspackPlugin({}), new Plugin()],
};
