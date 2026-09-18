/** @type {import("../../../../").Configuration} */
export default {
  target: 'web',
  module: {
    parser: {
      javascript: {
        dynamicUrl: false,
      },
    },
  },
};
