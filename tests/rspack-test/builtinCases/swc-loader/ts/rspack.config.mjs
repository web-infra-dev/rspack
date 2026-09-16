/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.tsx$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
              jsc: {
                target: 'es2015',
                parser: {
                  dynamicImport: true,
                  classProperty: true,
                  exportNamespaceFrom: true,
                  exportDefaultFrom: true,
                },
              },
            },
          },
        ],
        type: 'javascript/auto',
      },
    ],
  },
  devtool: 'inline-source-map',
};
