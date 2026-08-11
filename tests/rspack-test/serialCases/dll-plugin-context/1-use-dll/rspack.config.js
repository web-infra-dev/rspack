var webpack = require('@rspack/core');

const manifest = require('../../../js/config/dll-plugin-context/manifest0.json'); // eslint-disable-line node/no-missing-require
const camelCaseManifest = JSON.parse(JSON.stringify(manifest));
for (const item of Object.values(camelCaseManifest.content)) {
  if (item.buildMeta?.defaultObject === 'redirect-warn') {
    item.buildMeta.defaultObject = 'redirectWarn';
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
