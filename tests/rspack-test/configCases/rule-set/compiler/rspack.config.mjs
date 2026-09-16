/** @type {import("@rspack/core").Configuration} */
export default {
  name: 'compiler-name',
  module: {
    rules: [
      {
        test: /a\.js$/,
        compiler: 'compiler',
        use: './loader',
      },
      {
        test: /b\.js$/,
        compiler: 'other-compiler',
        use: './loader',
      },
    ],
  },
};
