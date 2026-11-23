import { defineConfig } from '@rspack/cli';

const library = { type: 'commonjs2' };

export default defineConfig({
  target: 'node',
  entry: {
    index: './index.js',
    'named-only': { import: './named-only.js', library },
    'with-default': { import: './with-default.js', library },
    'reexport-default': { import: './reexport-default.js', library },
    'with-require': { import: './with-require.js', library },
    'reexport-star': { import: './reexport-star.js', library },
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimize: false,
    concatenateModules: false,
  },
});
