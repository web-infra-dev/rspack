/** @type {import("../../../../").Configuration} */
module.exports = {
  module: {
    parser: {
      javascript: {
        exprContextCritical: true,
        wrappedContextCritical: true,
      }
    }
  },
  plugins: [
    function (compiler) {
      console.log(compiler.options)
    }
  ]
}
