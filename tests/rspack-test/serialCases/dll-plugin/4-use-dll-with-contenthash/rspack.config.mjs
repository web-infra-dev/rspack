import { rspack as webpack } from '@rspack/core';
import { readFileSync } from 'node:fs';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    moduleIds: 'named',
  },
  plugins: [
    new webpack.DllReferencePlugin({
      manifest: JSON.parse(
        readFileSync(
          new URL(
            '../../../js/config/dll-plugin/manifest0.json',
            import.meta.url,
          ),
          'utf-8',
        ),
      ),
      name: '../0-create-dll-with-contenthash/dll.js',
      scope: 'dll',
      sourceType: 'commonjs2',
      extensions: ['.js', '.jsx'],
    }),
  ],
};
