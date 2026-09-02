const path = require("path");
const { NormalModule } = require("@rspack/core");

const PLUGIN_NAME = "BeforeLoadersPlugin";
const tagLoader = require.resolve("./tag-loader.js");

class BeforeLoadersPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			NormalModule.getCompilationHooks(compilation).beforeLoaders.tap(
				PLUGIN_NAME,
				(loaders, normalModule) => {
					const name = path.basename(normalModule.userRequest);

					if (name === "untouched.js") {
						// The list handed to the hook mirrors what `module.rules` resolved,
						// options object included.
						if (loaders.length !== 1) {
							throw new Error(
								`expected one configured loader on untouched.js, got ${loaders.length}`
							);
						}
						if (loaders[0].loader !== tagLoader) {
							throw new Error(
								`expected the absolute loader path, got ${loaders[0].loader}`
							);
						}
						if (loaders[0].options.tag !== "+config") {
							throw new Error(
								`expected the configured options, got ${JSON.stringify(loaders[0].options)}`
							);
						}
					}

					if (name === "add.js") {
						loaders.push({
							loader: tagLoader,
							options: { tag: "+added" },
							ident: null,
							type: null
						});
					}

					if (name === "mutate.js") {
						loaders[0].options = { tag: "+mutated" };
					}

					if (name === "remove.js") {
						loaders.length = 0;
					}
				}
			);
		});
	}
}

module.exports = BeforeLoadersPlugin;
