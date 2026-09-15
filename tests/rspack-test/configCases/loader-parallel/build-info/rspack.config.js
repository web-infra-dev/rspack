const path = require('path');

class Plugin {
  apply(compiler) {
    compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
      const module = [...compilation.modules].find((m) =>
        m.resource?.endsWith('lib.js'),
      );
      const { buildInfo } = module;

      expect(buildInfo.parallel).toBe(true);

      expect(buildInfo.trail).toEqual(['main', 'worker']);

      expect(buildInfo.buildDependencies.has('./build.txt')).toBe(true);
    });
  }
}

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
  context: __dirname,
  module: {
    rules: [
      {
        test: /lib\.js/,
        use: [
          { loader: './worker-loader.js', parallel: true, options: {} },
          { loader: './seed-loader.js' },
        ],
      },
    ],
  },
  plugins: [new Plugin()],
};
