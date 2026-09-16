/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    target: 'web',
    output: {
      filename: 'bundle0.js',
      trustedTypes: true,
    },
    devtool: 'eval-source-map',
  },
  {
    target: 'web',
    output: {
      filename: 'bundle1.js',
      trustedTypes: true,
    },
    devtool: 'eval',
  },
];
