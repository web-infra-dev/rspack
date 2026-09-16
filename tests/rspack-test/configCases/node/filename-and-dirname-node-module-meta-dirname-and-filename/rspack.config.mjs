/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  output: {
    module: true,
    importMetaName: 'custom',
    environment: {
      importMetaDirnameAndFilename: true,
    },
  },
  node: {
    __filename: 'node-module',
    __dirname: 'node-module',
  },
};
