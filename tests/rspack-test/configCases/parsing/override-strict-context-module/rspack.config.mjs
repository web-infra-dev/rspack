/** @type {import("../../../../").Configuration} */
export default {
  mode: 'production',
  entry: ['./strict'],
  module: {
    parser: {
      javascript: {
        overrideStrict: 'strict',
      },
    },
  },
};
