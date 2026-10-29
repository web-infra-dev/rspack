import { defineConfig } from '@rspack/cli';
import fs from 'node:fs';
import path from 'node:path';

// a workspace package: installed as a symlink, so its own files resolve to a
// real path outside node_modules while the package request resolves through it
const modulesDirectory = path.resolve(
  import.meta.dirname,
  '../../../js/side-effects-symlinked/node_modules',
);

fs.mkdirSync(modulesDirectory, { recursive: true });

try {
  fs.symlinkSync(
    path.join(import.meta.dirname, 'package'),
    path.join(modulesDirectory, 'symlinked-sef'),
    'junction',
  );
} catch (err) {
  if ((err as NodeJS.ErrnoException).code !== 'EEXIST') throw err;
}

export default defineConfig({
  mode: 'production',
  resolve: {
    modules: [modulesDirectory, 'node_modules'],
  },
  optimization: {
    sideEffects: true,
    usedExports: true,
    providedExports: true,
    concatenateModules: false,
    minimize: false,
  },
});
