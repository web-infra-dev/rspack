import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  entry: {
    main: './index.js',
  },
  resolve: {
    tsConfig: {
      configFile: path.resolve(import.meta.dirname, './tsconfig.json'),
      references: 'auto',
    },
  },
});
