const { rspack } = require('@rspack/core');

module.exports = {
  output: {
    library: { type: 'commonjs-module' },
    importFunctionName: '((name) => Promise.resolve({ request: name }))',
  },
  externals: {
    fs: 'node-commonjs fs',
    'promise-external':
      'promise new Promise(resolve => setTimeout(() => resolve(42), 100))',
    'module-promise-external':
      'promise new Promise(resolve => setTimeout(() => resolve({ __esModule: true, default: 42, named: true }), 100))',
    'object-promise-external':
      'promise new Promise(resolve => setTimeout(() => resolve({ default: 42, named: true }), 100))',
    'failing-promise-external':
      "promise new Promise((resolve, reject) => setTimeout(() => reject(new Error('external reject')), 100))",
    'import-external': ['import /hello/world.js', 'request'],
    'module-import-external': ['module-import /hello/world.js', 'request'],
  },
  optimization: {
    inlineExports: true,
  },
  plugins: [
    new rspack.ProvidePlugin({
      providedAsyncModule: 'module-promise-external',
      providedAsyncModuleNamed: ['module-promise-external', 'named'],
      providedAsyncInlined: ['./provided-async-module.js', 'inlined'],
    }),
  ],
};
