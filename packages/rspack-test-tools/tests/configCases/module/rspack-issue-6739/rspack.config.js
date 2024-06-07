let foo = {}, bar = {}
module.exports = {
	module: {
		rules: [
			{
				test: require.resolve("foo"),
				loader: "./loader",
				descriptionData: {
					"componentId": (componentIdData) => {
						foo.componentIdData = componentIdData
						return true
					},
					"componentId.scope": (scopeData) => {
						foo.scopeData = scopeData
						return true
					}
				}
			},
			{
				test: require.resolve("bar"),
				loader: "./empty-loader",
				descriptionData: {
					"_custom_key": (customKey) => {
						bar.customKey = customKey
						// return `true` causes error:
						// `Error: didn't return a Buffer or String`
						return false
					},
				}
			}
		]
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tap("_", () => {
					expect(foo.componentIdData).toMatchObject({
							"scope": "react",
							"name": "examples/button",
							"version": "0.0.0"
					})
					expect(foo.scopeData).toBe("react")
					expect(bar.customKey).toBe(true)
				})
			}
		}
	]
}