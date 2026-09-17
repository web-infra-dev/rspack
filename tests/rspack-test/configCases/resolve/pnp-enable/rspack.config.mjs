/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  entry: {
    main: './index.js',
  },
  resolve: {
    pnp: true,
  },
};
