import { defineConfig, definePlugin } from '@rspack/cli';
import { type EntryDescription } from '@rspack/core';

const cases: { import: string; publicPath: EntryDescription['publicPath'] }[] =
  [
    {
      import: './a.js',
      publicPath: '/static/[hash:11]/',
    },
    {
      import: './a.js',
      publicPath: '/static/[fullhash:11]/',
    },
    {
      import: './a.js',
      publicPath: () => '/static/[hash:11]/',
    },
    {
      import: './a.js',
      publicPath: () => '/static/[fullhash:11]/',
    },
    {
      import: './a.js',
      publicPath: ({ hash }) => {
        return `/static/${hash!.slice(0, 11)}/`;
      },
    },
  ];
let bundleId = 1;

export default defineConfig({
  entry: cases.reduce<Record<string, EntryDescription>>((acc, c) => {
    acc[`bundle${bundleId++}`] = { ...c };
    return acc;
  }, {}),
  plugins: [
    definePlugin({
      apply(compiler) {
        const { EntryPlugin } = compiler.rspack;
        for (const c of cases) {
          new EntryPlugin(compiler.context, c.import, {
            name: `bundle${bundleId++}`,
            publicPath: c.publicPath,
          }).apply(compiler);
        }
      },
    }),
  ],
  output: {
    filename: '[name].js',
  },
});
