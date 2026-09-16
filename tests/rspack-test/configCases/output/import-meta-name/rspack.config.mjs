/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    importMetaName: 'pseudoImport.meta',
  },
  module: {
    parser: {
      javascript: {
        importMeta: false,
      },
    },
  },
};
