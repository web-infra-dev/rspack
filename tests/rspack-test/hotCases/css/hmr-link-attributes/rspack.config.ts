import { defineConfig } from '@rspack/cli';
import { RuntimePlugin } from '@rspack/core';

export default defineConfig({
  plugins: [
    (compiler) => {
      compiler.hooks.thisCompilation.tap('LinkAttributes', (compilation) => {
        RuntimePlugin.getCompilationHooks(compilation).createLink.tap(
          'LinkAttributes',
          (code) => `${code}\nlink.setAttribute("data-hook", "current");`,
        );
      });
    },
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
