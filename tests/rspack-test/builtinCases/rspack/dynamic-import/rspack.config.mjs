/** @type {import("@rspack/core").Configuration} */
export default {
  target: ['node'],
  entry: {
    main: {
      import: ['./index.js'],
    },
  },
};
