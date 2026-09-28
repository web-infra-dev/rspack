import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    './no-trusted-types-policy-name.web.js':
      'commonjs ./no-trusted-types-policy-name.web.js',
  },
  target: 'web',
  output: {
    // TODO should be `[name].web.js`
    chunkFilename: 'no-trusted-types-policy-name.web.js',
    crossOriginLoading: 'anonymous',
  },
  // performance: {
  // 	hints: false
  // },
  optimization: {
    minimize: false,
  },
});
