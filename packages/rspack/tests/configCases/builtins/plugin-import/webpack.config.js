module.exports = {
	module: {
		rules: [
			{
				test: /.css/,
				type: "asset"
			}
		]
	},
	builtins: {
		pluginImport: [
			{
				libraryName: "./src/foo",
				customName: "./src/foo/{{ kebabCase member }}",
				style: true
			},
			{
				libraryName: "./src/bar",
				customName: "./src/bar/{{ kebabCase member }}",
				style: `{{ member }}/style.css`
			}
		]
	}
};
