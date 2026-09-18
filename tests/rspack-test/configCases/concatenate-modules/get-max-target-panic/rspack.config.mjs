/**@type {import("@rspack/core").Configuration}*/
export default {
  mode: 'development',
  entry: {
    main: './index.js',
  },
  optimization: {
    providedExports: true,
    usedExports: true,
    concatenateModules: true,
    minimize: false,
  },
};
