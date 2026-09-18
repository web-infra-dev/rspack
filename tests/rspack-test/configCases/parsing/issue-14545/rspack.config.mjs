/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    unknownContextRegExp: /^\.\//,
    unknownContextCritical: false,
    exprContextRegExp: /^\.\//,
    exprContextCritical: false,
  },
};
