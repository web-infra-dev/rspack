/** @type {import("@rspack/core").Configuration} */
export default {
  target: [`async-node${process.versions.node.split('.').map(Number)[0]}`],
  entry: ['../defer-runtime/all.js'],
  experiments: {
    deferImport: true,
  },
};
