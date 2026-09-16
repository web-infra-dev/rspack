/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    mode: 'development',
  },
  {
    mode: 'production',
  },
  {
    mode: 'production',
    optimization: {
      concatenateModules: false,
    },
  },
  {
    mode: 'development',
    optimization: {
      concatenateModules: true,
    },
  },
];
