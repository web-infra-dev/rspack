const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: {
			import: ["./index"],
			library: {
				type: "system"
			}
		}
	},
	node: {
		__dirname: false,
		__filename: false
	},
	target: "web",
	output: {
		filename: "[name].js"
	},
	optimization: {
		sideEffects: false,
		concatenateModules: false,
		splitChunks: {
			cacheGroups: {
				default: {
					chunks: "all",
					minSize: 0,
					maxSize: 0
				}
			}
		}
	},
	plugins: [
		new rspack.ExternalsPlugin("system", ({ request }, callback) => {
			if (request === "./external") {
				callback(null, "system " + request);
				return;
			}
			callback();
		}),
		compiler => {
			compiler.hooks.afterEmit.tap("PLUGIN", compilation => {
				const stats = compilation.getStats().toJson();
				const entryChunk = stats.chunks.find(chunk => chunk.entry);
				expect(entryChunk.modules[0].name).toBe('external "./external"');
			});
		}
	]
};
