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
						import: [
							{
								libraryName: "./src/custom-name-tpl-camel",
								customName: "./src/custom-name-tpl-camel/{{ camelCase member }}"
							},
							{
								libraryName: "./src/custom-name-tpl-kebab",
								customName: "./src/custom-name-tpl-kebab/{{ kebabCase member }}"
							},
							{
								libraryName: "./src/custom-name-tpl-snake",
								customName: "./src/custom-name-tpl-snake/{{ snakeCase member }}"
							},
							{
								libraryName: "./src/custom-name-tpl-lower",
								customName: "./src/custom-name-tpl-lower/{{ lowerCase member }}"
							},
							{
								libraryName: "./src/custom-name-tpl-upper",
								customName: "./src/custom-name-tpl-upper/{{ upperCase member }}"
							},
							{
								libraryName: "./src/custom-style-name",
								customStyleName:
									"./src/custom-style-name/lib/{{ kebabCase member }}/style/index.css"
							},
							{
								libraryName: "./src/ignore-es-component",
								style: true,
								customStyleName:
									"./src/ignore-es-component/lib/{{ kebabCase member }}/style/index.css",
								ignoreEsComponent: ["FooBar"]
							},
							{
								libraryName: "./src/ignore-style-component",
								style: true,
								ignoreStyleComponent: ["FooBar"]
							},
							{
								libraryName: "./src/no-default",
								transformToDefaultImport: false
							},
							{
								libraryName: "./src/style-css",
								style: "css"
							},
							{
								libraryName: "./src/style-library",
								styleLibraryDirectory: "css"
							},
							{
								libraryName: "./src/style-tpl",
								style: "{{ member }}.css"
							},
							{
								libraryName: "./src/style-true",
								style: true
							},
							{
								libraryName: './src/legacy-babel-plugin-import',
								customName: './src/legacy-babel-plugin-import/lib/{{ legacyKebabCase member }}/{{ legacySnakeCase member }}',
								style: false
							}
						]
					}
				}
			}
		]
	}
};
