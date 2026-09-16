/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /[ab]\.js$/,
        use: function (data) {
          return {
            loader: './loader',
            // DIFF: need to use ident to identify the loader options
            ident: `${data.issuer}|${data.resource}?${data.resourceQuery}`,
            options: {
              resource: data.resource.replace(/^.*[\\/]/g, ''),
              resourceQuery: data.resourceQuery,
              issuer: data.issuer.replace(/^.*[\\/]/g, ''),
            },
          };
        },
      },
    ],
  },
};
