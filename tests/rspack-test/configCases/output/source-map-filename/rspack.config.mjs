/** @type {import("@rspack/coresrc/index").RspackOptions} */
export default {
  entry: {
    main: './index',
  },
  devtool: 'source-map',
  target: 'node',
  output: {
    filename: '[name].js',
    sourceMapFilename: '../maps/[name].js.map',
  },
};
