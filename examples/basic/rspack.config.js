/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
  entry: {
    main: './src/index.js'
  },
  builtins: {
    html: [
      {
        template: './index.html'
      }
    ]
  }
}