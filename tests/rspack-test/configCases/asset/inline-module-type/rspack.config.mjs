import { fileURLToPath } from 'node:url';
export default {
  module: {
    rules: [
      {
        test: /\.txt$/,
        type: 'asset/source',
        resolve: {
          alias: { value: fileURLToPath(import.meta.resolve('./value.js')) },
        },
      },
    ],
  },
};
