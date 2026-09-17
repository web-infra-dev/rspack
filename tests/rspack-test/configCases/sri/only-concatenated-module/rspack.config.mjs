import { rspack } from '@rspack/core';
import path from 'node:path';
import fs from 'node:fs';

/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  externals: {
    path: "require('path')",
    fs: "require('fs')",
  },
  node: {
    __dirname: false,
  },
  output: {
    crossOriginLoading: 'anonymous',
  },
  optimization: {
    concatenateModules: true,
    minimize: false,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  plugins: [
    new rspack.SubresourceIntegrityPlugin(),
    {
      apply(compiler) {
        compiler.hooks.done.tap('TestPlugin', () => {
          const mainPath = path.join(
            compiler.options.output.path,
            'bundle0.js',
          );
          const mainContent = fs.readFileSync(mainPath, 'utf-8');
          expect(mainContent).toContain('.sriHashes = {"chunk": "sha384-');
        });
      },
    },
  ],
};
