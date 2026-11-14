import { defineConfig } from '@rspack/cli';

export default defineConfig(
  ['index.js', 'resolve.mjs', 'eager.mjs'].flatMap((entry, index) =>
    [false, true].map((minimize) => {
      const name = `${index}-${minimize ? 'min' : 'readable'}`;
      return defineConfig({
        name,
        target: 'node',
        entry: `./${entry}`,
        externalsType: 'module',
        externals: ['./ext.mjs', './side.mjs'],
        output: {
          filename: `${name}/[name].mjs`,
          chunkFilename: `${name}/[name].mjs`,
        },
        optimization: {
          minimize,
          splitChunks: {
            chunks: 'all',
            cacheGroups: {
              setup: {
                test: /setup\.mjs$/,
                name: 'setup',
                enforce: true,
              },
            },
          },
        },
      });
    }),
  ),
);
