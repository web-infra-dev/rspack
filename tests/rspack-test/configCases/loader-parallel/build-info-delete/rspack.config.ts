import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
    const { NormalModule } = compiler.rspack;

    compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
      const moduleOf = (name: string) =>
        [...compilation.modules].find(
          (m) => m instanceof NormalModule && m.resource?.endsWith(name),
        );

      const one = moduleOf('lib.js')!.buildInfo;
      expect(Object.hasOwn(one, 'dropMe')).toBe(false);
      expect(one.keep).toBe('kept');
      expect(one.added).toBe('added');

      const all = moduleOf('lib2.js')!.buildInfo;
      expect(Object.hasOwn(all, 'onlyKey')).toBe(false);
    });
  }
}

export default defineConfig({
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
});
