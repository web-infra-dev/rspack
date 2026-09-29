import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /[ab]\.js$/,
        use: function (data) {
          return {
            loader: './loader.mjs',
            // DIFF: need to use ident to identify the loader options
            ident: `${data.issuer}|${data.resource}?${data.resourceQuery}`,
            options: {
              resource: data.resource?.replace(/^.*[\\/]/g, ''),
              resourceQuery: data.resourceQuery,
              issuer: data.issuer.replace(/^.*[\\/]/g, ''),
            },
          };
        },
      },
    ],
  },
});
