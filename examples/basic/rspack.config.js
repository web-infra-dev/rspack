/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
  mode: 'development',
  entry: {
    main: './src/index.js'
  },
  builtins: {
    treeShaking: true
  },
  stats: {
    all: true
  }
}