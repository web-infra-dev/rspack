/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    module: true,
    library: {
      type: 'module',
    },
    enabledLibraryTypes: ['module', 'module'],
  },
  target: ['es2022'],
};
