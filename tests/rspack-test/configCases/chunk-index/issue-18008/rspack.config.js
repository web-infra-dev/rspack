/** @typedef {import("@rspack/core").Compilation} Compilation */
/** @typedef {import("@rspack/core").Module} Module */
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./main.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: false,
		chunkIds: "named"
	},
	plugins: [
		function () {
			/**
			 * @param {Compilation} compilation compilation
			 * @returns {void}
			 */
			const handler = compilation => {
				compilation.hooks.afterSeal.tap("testcase", () => {
					const data = {};
					for (const [name, group] of compilation.namedChunkGroups) {
						/** @type {Map<Module, number>} */
						const modules = new Map();
						for (const chunk of group.chunks) {
							for (const module of compilation.chunkGraph.getChunkModulesIterable(
								chunk
							)) {
								const preOrder = group.getModulePreOrderIndex(module);
								if (typeof preOrder === "number") {
									modules.set(module, preOrder);
								}
							}
						}
						const sortedModules = Array.from(modules).sort(
							(a, b) => a[1] - b[1]
						);
						const text = sortedModules
							.map(
								([m, index]) =>
									`${index}: ${m.readableIdentifier(
										compilation.requestShortener
									)}`
							)
							.join(", ");
						data[`${name}Index`] = text;
					}
					expect(data).toEqual({
						// DIFF: rspack will not generate prefix 'css' for readable identifiers of css modules.
						AIndex: "0: ./A.js, 1: ./m.css",
						"B-2Index": "0: ./B-2.js",
						BIndex: "0: ./B.js",
						mainIndex: "0: ./main.js",
						// DIFF:
						// css modules in rspack will only have one source types without css-export in it.
						// So the module 'n.css' will be disconnected from chunk in concatenated plugin
						// sharedIndex: "1: css ./m.css, 2: css ./n.css"
						sharedIndex: "1: ./m.css"
					});
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	],
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	}
};
