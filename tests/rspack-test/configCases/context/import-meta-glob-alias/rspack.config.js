const path = require('node:path');

module.exports = {
  context: __dirname,
  resolve: {
    alias: {
      dir: [],
      ignored: false,
      nested$: path.join(__dirname, 'src', 'does-not-exist'),
      '@': path.join(__dirname, 'src'),
      '@meta': path.join(__dirname, 'src/[aliased]/{assets}'),
      '@relocated': path.join(__dirname, 'src/glob-imports'),
      '~': __dirname,
      'special-dir': path.join(__dirname, 'src', '[dir]'),
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.contextModuleFactory.tap(
          'RelocateGlobRoot',
          (factory) => {
            factory.hooks.afterResolve.tap('RelocateGlobRoot', (data) => {
              if (data && data.request.includes('@relocated')) {
                data.resource = path.join(__dirname, 'src/[aliased]/{assets}');
              }
            });
          },
        );
      },
    },
  ],
};
