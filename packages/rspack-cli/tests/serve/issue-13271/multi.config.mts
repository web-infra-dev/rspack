import { defineConfig } from '../../../dist/index.js';
import statePlugin from './state-plugin.mts';

export default defineConfig([
  {
    name: 'disabled',
    mode: 'development',
    entry: './entry-disabled.js',
    devServer: false,
    plugins: [statePlugin('disabled')],
  },
  {
    name: 'web',
    mode: 'development',
    entry: './entry-web.js',
    devServer: {},
    plugins: [statePlugin('web')],
  },
]);
