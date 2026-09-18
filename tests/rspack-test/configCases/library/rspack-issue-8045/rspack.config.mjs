/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: {
      name: 'MyLibrary',
      export: 'default',
      type: 'window',
    },
  },
};
