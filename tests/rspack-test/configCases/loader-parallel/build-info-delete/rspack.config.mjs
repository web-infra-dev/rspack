class Plugin {
  apply(compiler) {
    compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
      const moduleOf = (name) =>
        [...compilation.modules].find((m) => m.resource?.endsWith(name));

      const one = moduleOf('lib.js').buildInfo;
      expect(Object.hasOwn(one, 'dropMe')).toBe(false);
      expect(one.keep).toBe('kept');
      expect(one.added).toBe('added');

      const all = moduleOf('lib2.js').buildInfo;
      expect(Object.hasOwn(all, 'onlyKey')).toBe(false);
    });
  }
}

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib\.js/,
        use: [
          { loader: './worker-delete-loader.mjs', parallel: true, options: {} },
          { loader: './seed-loader.mjs' },
        ],
      },
      {
        test: /lib2\.js/,
        use: [
          {
            loader: './worker-delete-all-loader.mjs',
            parallel: true,
            options: {},
          },
          { loader: './seed-single-loader.mjs' },
        ],
      },
    ],
  },
  plugins: [new Plugin()],
};
