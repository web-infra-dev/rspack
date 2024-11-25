/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: {
    main: './index.js',
  },
  module: {
		generator: {
			'css/module': {
				exportsOnly: true
			}
		},
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
        parser: {
          namedExports: false,
        }
      }
    ]
  }
}
