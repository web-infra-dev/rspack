import { rspack as webpack } from '@rspack/core';
import { readFileSync } from 'node:fs';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new webpack.DllReferencePlugin({
      manifest: JSON.parse(
        readFileSync(
          new URL(
            '../../../js/config/dll-plugin-side-effects/manifest0.json',
            import.meta.url,
          ),
          'utf-8',
        ),
      ), // eslint-disable-line node/no-missing-require
      name: '../0-create-dll/dll.js',
      scope: 'dll',
      sourceType: 'commonjs2',
    }),
  ],
};
