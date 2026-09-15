const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration[]} */
module.exports = ['', 'auto', '/build-time/'].flatMap((publicPath) =>
  [undefined, false, true].flatMap((runtimePublicPath) =>
    [false, true].flatMap((concatenateModules) =>
      ['css', 'css/auto', 'css/global', 'css/module'].map((moduleType) => ({
        target: 'web',
        mode: 'production',
        devtool: false,
        entry: ['./public-path.js', './index.js'],
        output: {
          publicPath,
          assetModuleFilename: 'assets/[name][ext]',
        },
        optimization: { concatenateModules },
        module: {
          parser: { [moduleType]: { exportType: 'style', runtimePublicPath } },
          rules: [
            {
              test: /\.css$/,
              type: moduleType,
              ...(moduleType === 'css'
                ? {}
                : { generator: { localIdentName: '[local]' } }),
            },
            {
              test: /\.svg$/,
              type: 'asset/resource',
              generator: { outputPath: 'emitted/' },
            },
            {
              resourceQuery: /fixed/,
              type: 'asset/resource',
              generator: { publicPath: 'https://fixed.example.com/' },
            },
            { resourceQuery: /inline/, type: 'asset/inline' },
            ...['text', 'css-style-sheet'].map((exportType) => ({
              resourceQuery: (query) => query === '?' + exportType,
              type: 'css',
              parser: { exportType, runtimePublicPath: true },
            })),
          ],
        },
        plugins: [
          new rspack.DefinePlugin({
            RUNTIME_PUBLIC_PATH: JSON.stringify(runtimePublicPath === true),
            BUILD_PREFIX: JSON.stringify(
              publicPath === 'auto' ? '' : publicPath,
            ),
            ASSET_PREFIX: JSON.stringify(
              runtimePublicPath
                ? 'https://test.cases/path/'
                : publicPath === 'auto'
                  ? ''
                  : publicPath,
            ),
          }),
        ],
      })),
    ),
  ),
);
