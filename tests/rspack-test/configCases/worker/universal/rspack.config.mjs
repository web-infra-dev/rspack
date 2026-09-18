/** @type {import("@rspack/core").Configuration} */
export default [
  {
    name: 'web',
    target: ['web', 'node'],
    output: {
      module: true,
      filename: 'web-[name].mjs',
    },
  },
];
