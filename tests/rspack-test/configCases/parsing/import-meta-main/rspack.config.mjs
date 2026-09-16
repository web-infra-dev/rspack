/** @type {import("@rspack/core").Configuration} */
export default [true, false].map((concatenateModules) => {
  return {
    target: 'node',
    optimization: {
      concatenateModules,
    },
  };
});
