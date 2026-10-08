import { readFileSync } from 'node:fs';
import { defineConfig } from '@rspack/cli';
import { type DllReferencePluginOptionsManifest, rspack } from '@rspack/core';

const manifest: DllReferencePluginOptionsManifest = JSON.parse(
  readFileSync(
    new URL(
      '../../../js/config/dll-plugin-context/manifest0.json',
      import.meta.url,
    ),
    'utf-8',
  ),
);
const camelCaseManifest: DllReferencePluginOptionsManifest = JSON.parse(
  JSON.stringify(manifest),
);
for (const item of Object.values(camelCaseManifest.content)) {
  if (item.buildMeta?.defaultObject === 'redirect-warn') {
    // @ts-expect-error Exercise legacy manifest input outside the public type.
    item.buildMeta.defaultObject = 'redirectWarn';
  }
}

export default defineConfig({
  optimization: {
    moduleIds: 'named',
  },
  plugins: [
    new rspack.DllReferencePlugin({
      manifest,
      name: '../0-create-dll/dll.js',
      scope: 'dll',
      sourceType: 'commonjs2',
    }),
    new rspack.DllReferencePlugin({
      // Rspack 2.x used to emit the camelCase spelling. Keep accepting those
      // manifests while new manifests use webpack's kebab-case spelling.
      manifest: camelCaseManifest,
      name: '../0-create-dll/dll.js',
      scope: 'camel-dll',
      sourceType: 'commonjs2',
    }),
  ],
});
