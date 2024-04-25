const table = {
	['main1']: 'main1',
	['main2']: 'main2-runtime'
}

function plugin(compiler) {
	compiler.hooks.compilation.tap("plugin", compilation => {
		compilation.hooks.processAssets.tap("plugin", () => {
			for (let [name, entrypoint] of compilation.entrypoints.entries()) {
				const runtimeChunk = entrypoint.getRuntimeChunk();
				expect(runtimeChunk.name).toBe(table[name])
			}
		});
	});
}

const common = {
	output: {
		filename: "[name].js",
	},
	plugins: [plugin]
}

module.exports = [{
	...common,
	entry: {
		main1: "./entry1.js",
	},
}, {
	...common,
	entry: {
		main2: {
			import: "./entry2.js",
			runtime: "main2-runtime"
		}
	},
}];
