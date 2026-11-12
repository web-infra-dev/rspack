import { defineConfig } from '@rspack/cli';
import fs from 'node:fs';
import path from 'node:path';

// a package without `sideEffects` whose `shared` directory links to a directory inside
// a package that declares `sideEffects: false`, imported relatively from the
// package's own module
const packageDirectory = path.resolve(
  import.meta.dirname,
  '../../../js/side-effects-symlinked-directory',
);

fs.mkdirSync(packageDirectory, { recursive: true });
fs.writeFileSync(
  path.join(packageDirectory, 'package.json'),
  '{"name":"symlinked-directory-importer"}',
);
fs.writeFileSync(
  path.join(packageDirectory, 'index.js'),
  'import "./shared/side.js";\n',
);

try {
  fs.symlinkSync(
    path.join(import.meta.dirname, 'lib', 'src'),
    path.join(packageDirectory, 'shared'),
    'junction',
  );
} catch (err) {
  if ((err as NodeJS.ErrnoException).code !== 'EEXIST') throw err;
}

export default defineConfig({
  mode: 'production',
  resolve: {
    alias: {
      'symlinked-directory-importer$': path.join(packageDirectory, 'index.js'),
    },
  },
  optimization: {
    sideEffects: true,
    minimize: false,
  },
});
