import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
