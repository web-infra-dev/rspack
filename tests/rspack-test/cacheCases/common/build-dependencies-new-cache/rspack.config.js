const path = require("path");

const buildDependency = path.join(__dirname, "./configs/index.js");
let compilerIndex = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	experiments: {
		newCache: true
	},
	cache: {
		type: "persistent",
		buildDependencies: [buildDependency]
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tap("Test Plugin", stats => {
					const logging = stats.toJson({
						all: false,
						logging: "verbose"
					}).logging;
					const entries = logging["rspack.Compilation"]?.entries ?? [];
					const cacheEntry = entries.find(
						entry =>
							entry.type === "cache" &&
							entry.message?.startsWith("module code generation cache:")
					);
					expect(cacheEntry).toBeTruthy();

					const match = cacheEntry.message.match(/\((\d+)\/(\d+)\)/);
					expect(match).toBeTruthy();
					const hits = Number(match[1]);
					const total = Number(match[2]);
					expect(total).toBeGreaterThan(0);

					if (compilerIndex === 1 || compilerIndex === 4) {
						expect(hits).toBe(total);
					} else {
						expect(hits).toBe(0);
					}
					compilerIndex++;
				});
			}
		}
	]
};
