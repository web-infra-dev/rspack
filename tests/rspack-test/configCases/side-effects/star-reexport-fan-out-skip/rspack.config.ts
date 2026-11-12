import { defineConfig } from '@rspack/cli';

const optimization = {
  sideEffects: true,
  usedExports: true,
  providedExports: true,
  minimize: false,
};

export default defineConfig([
  {
    mode: 'production',
    output: { filename: 'bundle0.js' },
    optimization: { ...optimization, concatenateModules: false },
  },
  {
    mode: 'production',
    output: { filename: 'bundle1.js' },
    optimization: { ...optimization, concatenateModules: true },
  },
]);
