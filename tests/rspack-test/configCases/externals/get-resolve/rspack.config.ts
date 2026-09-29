import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  externals: [
    ({ context, request, getResolve }, callback) => {
      const resolveFunction = getResolve!();
      resolveFunction(context!, request!, (err, resource) => {
        if (err) {
          return callback(err);
        }
        if (
          resource ===
          path.resolve(import.meta.dirname, 'node_modules/foo/index.mjs')
        ) {
          callback(undefined, 'global esm');
        } else if (
          resource ===
          path.resolve(import.meta.dirname, 'node_modules/foo/index.cjs')
        ) {
          callback(undefined, 'global cjs');
        } else {
          callback(undefined, false);
        }
      });
    },
  ],
});
