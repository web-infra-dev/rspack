module.exports = {
	entry: {
		main: './index.js',
	},
	module: {
    parser: {
      javascript: {
        inlineEnum: true,
      },
    },
    rules: [
      {
        test: /\.ts/,
        sideEffects: false,
        use: [
          {
            loader: "builtin:swc-loader",
            options: {
              jsc: {
                parser: {
                  syntax: "typescript",
                }
              },
              rspackExperiments: {
                collectTypeScriptInfo: {
                  exportedEnum: true,
                },
              },
            },
          },
        ],
      },
    ],
	},
	optimization: {
		concatenateModules: true,
	},
	experiments: {
		inlineEnum: true,
	}
}
