/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    parser: {
      javascript: {
        exprContextCritical: true,
        wrappedContextCritical: true,
      }
    }
  },
}
