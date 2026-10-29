import { defineConfig } from '../../../../dist/index.js';

export default defineConfig({
  entry: './index.js',
  context: './context', // relative path will fail
  _additionalProperty: 'test',
});
