import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    assetModuleFilename: ({ filename }) => {
      if (/.png$/.test(filename!)) {
        return 'images/[\\ext\\]/success-png[ext]';
      }
      if (/.svg$/.test(filename!)) {
        return 'images/success-svg[ext]';
      }
      return 'images/failure[ext]';
    },
  },
  module: {
    rules: [
      {
        test: /\.(png|svg)$/,
        type: 'asset/resource',
        rules: [
          {
            resourceQuery: '?custom2',
            generator: {
              filename: 'custom-images/success[ext]',
            },
          },
        ],
      },
    ],
  },
});
