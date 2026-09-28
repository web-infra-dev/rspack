import { rspack as webpack } from '@rspack/core';
import { readFileSync } from 'node:fs';

const manifest = JSON.parse(
  readFileSync(
    new URL(
      '../../../js/config/dll-plugin-context/manifest0.json',
      import.meta.url,
    ),
    'utf-8',
  ),
); // eslint-disable-line node/no-missing-require
const camelCaseManifest = JSON.parse(JSON.stringify(manifest));
for (const item of Object.values(camelCaseManifest.content)) {
  if (item.buildMeta?.defaultObject === 'redirect-warn') {
    item.buildMeta.defaultObject = 'redirectWarn';
  }
}

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    moduleIds: 'named',
  },
  plugins: [
    new webpack.DllReferencePlugin({
      manifest,
      name: '../0-create-dll/dll.js',
      scope: 'dll',
      sourceType: 'commonjs2',
    }),
    new webpack.DllReferencePlugin({
      // Rspack 2.x used to emit the camelCase spelling. Keep accepting those
      // manifests while new manifests use webpack's kebab-case spelling.
      manifest: camelCaseManifest,
      name: '../0-create-dll/dll.js',
      scope: 'camel-dll',
      sourceType: 'commonjs2',
    }),
  ],
};
