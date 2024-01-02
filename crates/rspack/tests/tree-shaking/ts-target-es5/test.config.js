module.exports = {
  "target": [
    "web",
    "es5"
  ],
	module: {
		rules: [
			{
				test: /\.(j|t)s$/,
				exclude: [/[\\/]node_modules[\\/]/],
				loader: "builtin:swc-loader",
				options: {
					sourceMap: false,
					jsc: {
						parser: {
							syntax: "typescript"
						},
						externalHelpers: true
					},
					env: {
						targets: "Chrome >= 48"
					}
				}
			},
			{
				test: /\.(j|t)sx$/,
				loader: "builtin:swc-loader",
				exclude: [/[\\/]node_modules[\\/]/],
				options: {
					sourceMap: false,
					jsc: {
						parser: {
							syntax: "typescript",
							tsx: true
						},
						transform: {
							react: {
								runtime: "automatic",
							}
						},
						externalHelpers: true
					},
					env: {
						targets: "Chrome >= 48"
					}
				}
			},
		]
	},
  "optimization": {
    "sideEffects": "true"
  },
  "builtins": {
    "treeShaking": "true",
    "define": {
      "process.env.NODE_ENV": "'development'"
    }
  }
}
