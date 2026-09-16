/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    noParse: function (content) {
      return /not-parsed/.test(content);
    },
  },
};
