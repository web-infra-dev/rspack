/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  output: {
    module: true,
    importMetaName: 'custom',
  },
  node: {
    __filename: 'node-module',
    __dirname: 'node-module',
  },
};
