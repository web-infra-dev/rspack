/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  output: {
    assetModuleFilename: 'images/failure[ext]',
  },
  module: {
    rules: [
      {
        test: /\.(png|jpg)$/,
        type: 'asset/resource',
        rules: [
          {
            resourceQuery: '?custom1',
            generator: {
              filename: 'custom-images/success1[ext]',
            },
          },

          {
            resourceQuery: '?custom2',
            generator: {
              filename: ({ filename }) => {
                if (filename.endsWith('.png?custom2')) {
                  return 'custom-images/success2[ext]';
                }
                return 'images/failure2[ext]';
              },
            },
          },
          {
            resourceQuery: '?custom3',
            generator: {
              filename:
                '模板/[name]-[contenthash:8]-[contenthash:16]-[contenthash:8]-[contenthash:o]-[hash:1234]-[contenthash:base64:4][ext]',
            },
          },
        ],
      },
    ],
  },
};
