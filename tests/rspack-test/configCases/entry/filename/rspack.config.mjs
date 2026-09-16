/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: {
      import: './index.js',
      filename: 'my-[name].js',
    },
  },
};
