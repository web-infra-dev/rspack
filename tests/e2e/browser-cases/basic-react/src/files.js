import { BrowserHttpImportEsmPlugin } from '@rspack/browser';

export const files = {
  './src/main.ts': `
  const rspack: string = "rspack";
  console.log(rspack);`,
};

export const config = {
  mode: 'development',
  devtool: false,
  entry: './src/main.ts',
  module: {
    rules: [
      {
        test: /\.ts$/,
        use: {
          loader: 'builtin:swc-loader',
          options: {
            jsc: {
              parser: {
                syntax: 'typescript',
              },
            },
          },
        },
        type: 'javascript/auto',
      },
    ],
  },
};
