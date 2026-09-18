/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    output: {
      globalObject: "null || new Function('return this')()",
    },
  },
  {
    output: {
      globalObject: "(new Function('return this'))()",
    },
  },
  {
    output: {
      globalObject: "1 > 2 ? null : new Function('return this')()",
    },
  },
];
