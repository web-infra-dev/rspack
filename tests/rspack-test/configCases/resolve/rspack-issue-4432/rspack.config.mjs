/** @type {import("@rspack/core").Configuration} */
const config = {
  entry: {
    main: './index.js',
  },
  resolve: {
    mainFields: ['custom', '...'],
  },
};
export default config;
