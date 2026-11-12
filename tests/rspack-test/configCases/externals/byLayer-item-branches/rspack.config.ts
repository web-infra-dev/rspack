import { defineConfig } from '@rspack/cli';
import type { ExternalItem } from '@rspack/core';

const branches: ExternalItem[] = [
  'path',
  /^path$/,
  ({ request }, callback) => {
    callback(null, request === 'path' ? 'commonjs path' : undefined);
  },
  async ({ request }) => (request === 'path' ? 'commonjs path' : undefined),
];

export default defineConfig(
  branches.flatMap((branch) =>
    [false, true].map((dynamic) => ({
      externalsPresets: { node: false },
      externalsType: 'commonjs',
      externals: { byLayer: dynamic ? () => branch : { app: branch } },
      module: { rules: [{ test: /app\.js$/, layer: 'app' }] },
    })),
  ),
);
