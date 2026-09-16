/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'none',
  target: 'node',
  node: {
    __dirname: false,
    __filename: false,
  },
  output: {
    filename: '[name].js',
    workerPublicPath: '/workerPublicPath2/',
  },
};
