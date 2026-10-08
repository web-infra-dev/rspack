const { rspack } = require('@rspack/core');

module.exports = [true, 'relative', 'new-url-relative']
  .flatMap((url) => [false, true].map((extractCss) => ({ url, extractCss })))
  .map(({ url, extractCss }, index) => {
    const name = `${url === true ? 'default' : url}-${extractCss ? 'split' : 'control'}`;
    const outputModule = url === 'new-url-relative';
    const extension = outputModule ? 'mjs' : 'js';
    return {
      name,
      mode: 'development',
      devtool: false,
      target: 'web',
      externals: {
        fs: 'node-commonjs fs',
        path: 'node-commonjs path',
      },
      output: {
        module: outputModule,
        filename: `bundle${index}.${extension}`,
        chunkFilename: `${name}-[name].${extension}`,
        cssFilename: `${name}-[name].css`,
        cssChunkFilename: `${name}-[name].css`,
        publicPath: '/assets/',
      },
      module: {
        parser: { javascript: { url } },
        rules: [
          { test: /target\.css$/, dependency: 'url', type: 'css' },
          { test: /target-imported\.css$/, type: 'css' },
        ],
      },
      optimization: {
        splitChunks: extractCss
          ? {
              chunks: 'all',
              cacheGroups: {
                default: false,
                defaultVendors: false,
                styles: {
                  test: /\.css$/,
                  name: 'shared',
                  enforce: true,
                },
              },
            }
          : false,
      },
      plugins: [
        (compiler) => {
          new rspack.DefinePlugin({
            OUTPUT_DIR: JSON.stringify(compiler.options.output.path),
          }).apply(compiler);
        },
        new rspack.DefinePlugin({
          EXTRACT_CSS: JSON.stringify(extractCss),
          CONFIG_NAME: JSON.stringify(name),
        }),
      ],
    };
  });
