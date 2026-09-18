/** @type {import("@rspack/core").Configuration} */
export default {
  devtool: 'source-map',
  output: {
    library: ['Foo', '[name]'],
  },
};
