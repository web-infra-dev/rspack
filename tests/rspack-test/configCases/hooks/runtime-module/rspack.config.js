class Plugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("TestFakePlugin", compilation => {
			compilation.hooks.runtimeModule.tap("TestFakePlugin", (module, chunk) => {
				if (module.name === "has_own_property" && chunk.name === "main") {
					const originSource = module.source.source.toString("utf-8");
					module.source.source = Buffer.from(
						`${originSource}\n__webpack_require__.test = true;\n`,
						"utf-8"
					);
				}
			});
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	entry: {
		main: "./index.js",
		chunk: "./chunk.js"
	},
	output: {
		filename: "[name].js"
	},
	context: __dirname,
	plugins: [new Plugin()]
};
