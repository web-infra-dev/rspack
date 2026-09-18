/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    environment: {
      arrowFunction: true,
      bigIntLiteral: false,
      const: false,
      destructuring: false,
      forOf: false,
      dynamicImport: true,
      module: false,
    },
  },
};
