/**@type {import("@rspack/core").Configuration}*/
export default {
  context: import.meta.dirname,
  optimization: {
    innerGraph: true,
    sideEffects: true,
    usedExports: true,
    providedExports: true,
  },
};
