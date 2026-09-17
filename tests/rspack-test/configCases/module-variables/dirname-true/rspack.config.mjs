/** @type {import("@rspack/coresrc/index").RspackOptions} */
export default {
  context: import.meta.dirname,
  entry: {
    main: './index',
  },
  node: {
    __dirname: true,
    __filename: true,
  },
};
