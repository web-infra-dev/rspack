import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.(png|svg|jpg)$/,
        type: 'asset',
        parser: {
          dataUrlCondition: (
            _source: Buffer,
            { filename }: { filename: string },
          ) => {
            return filename.includes('?foo=bar');
          },
        },
      },
    ],
  },
});
