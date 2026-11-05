import { defineConfig } from '../../../dist/index.js';

export default defineConfig(function (env) {
  console.log(
    ['RSPACK_BUNDLE', 'RSPACK_BUILD', 'RSPACK_WATCH']
      .map((key) => `${key}=${env[key]}`)
      .join('\n'),
  );
  return {
    mode: 'development',
    entry: './src/entry.js',
  };
});
