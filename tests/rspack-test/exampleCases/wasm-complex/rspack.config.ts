import { defineConfig } from '@rspack/cli';

export default defineConfig({
  // mode: "development" || "production",
  output: {
    publicPath: 'dist/',
  },
  module: {
    rules: [
      {
        test: /\.wat$/,
        use: 'wast-loader',
        type: 'webassembly/async',
      },
    ],
  },
  optimization: {
    chunkIds: 'deterministic', // To keep filename consistent between different modes (for example building only)
  },
  experiments: {
    asyncWebAssembly: true,
  },
});
