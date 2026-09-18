import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: ['./index.js'],
  optimization: {
    // MAYBE: support ModuleConcatenationPlugin
    concatenateModules: true,
  },
  plugins: [
    new webpack.DllPlugin({
      path: path.resolve(
        import.meta.dirname,
        '../../../js/config/scope-hoisting/create-dll-plugin/manifest.json',
      ),
    }),
    // new webpack.optimize.ModuleConcatenationPlugin()
  ],
};
