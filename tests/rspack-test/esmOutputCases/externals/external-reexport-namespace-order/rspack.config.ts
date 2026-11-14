import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [
    ['remapped', 'index.js'],
    ['with-space', 'with-space.mjs'],
    ['commonjs', 'commonjs.mjs'],
    ['attributes', 'attributes.mjs'],
  ].flatMap(([variant, entry]) =>
    [false, true].map((minimize) => {
      const name = `${variant}-${minimize ? 'min' : 'readable'}`;
      return defineConfig({
        name,
        target: 'node',
        entry: `./${entry}`,
        externalsType: 'module',
        externals: {
          './first.mjs':
            variant === 'commonjs' ? './first.mjs' : ['./first.mjs', 'nested'],
          './first with space.mjs': ['./first with space.mjs', 'nested'],
          './数据 with space.json': ['./数据 with space.json', 'default'],
          './attribute-ext.mjs': './attribute-ext.mjs',
          './ext.mjs': './ext.mjs',
        },
        output: {
          filename: `${name}/[name].mjs`,
          chunkFilename: `${name}/[name].mjs`,
        },
        optimization: { minimize, runtimeChunk: false },
      });
    }),
  ),
);
