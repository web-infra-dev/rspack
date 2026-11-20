import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  resolve: {
    // A relative path is resolved from the working directory.
    packageMap: path.relative(
      process.cwd(),
      path.resolve(import.meta.dirname, 'package-map.json'),
    ),
    modules: ['node_modules', path.resolve(import.meta.dirname, 'modules')],
  },
});
