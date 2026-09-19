import { defineConfig } from '@rspack/cli';

import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

export default defineConfig({
  entry: {
    a: './a',
    b: './b',
  },
  output: {
    filename: 'MyDll.[name].js',
    library: '[name]_[fullhash]',
  },
  plugins: [
    new webpack.DllPlugin({
      path: path.resolve(
        import.meta.dirname,
        '../../../js/config/dll-plugin/manifest_without_string_template.json',
      ),
    }),
  ],
});
