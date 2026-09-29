/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.(png|svg|jpg)$/,
        type: 'asset',
        parser: {
          dataUrlCondition: (source, { filename, module }) => {
            return filename.includes('?foo=bar');
          },
        },
      },
    ],
  },
};
