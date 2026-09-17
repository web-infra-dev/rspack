/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: {
      name: 'named-system-module',
      type: 'system',
    },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
};
