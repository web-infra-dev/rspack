/** @type {import("@rspack/core").Configuration} */
module.exports = {
	resolve: {},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "asset"
			},
			{
				test: /\.js$/,
				loader: "builtin:swc-loader",
				options: {
					rspackExperiments: {
						transformImport: [
							{
								source: "./src/basic",
								output: ["./src/basic/lib/{{ kebabCase filename }}"]
							},
							{
								source: "./src/with-css",
								output: [
									"./src/with-css/es/{{ kebabCase filename }}.js",
									"./src/with-css/css/{{ kebabCase filename }}.css"
								]
							},
							{
								source: "./src/named-import",
								namedImport: true,
								output: ["./src/named-import/lib/{{ filename }}"]
							},
							{
								source: "./src/default-import",
								namedImport: false,
								output: ["./src/default-import/lib/{{ kebabCase filename }}"]
							},
							{
								source: "./src/exclude",
								output: ["./src/exclude/lib/{{ kebabCase filename }}"],
								exclude: ["Excluded"]
							},
							{
								source: "./src/helpers",
								output: [
									"./src/helpers/{{ camelCase filename }}/{{ snakeCase filename }}/{{ upperCase filename }}/{{ lowerCase filename }}"
								]
							}
						]
					}
				}
			}
		]
	}
};
