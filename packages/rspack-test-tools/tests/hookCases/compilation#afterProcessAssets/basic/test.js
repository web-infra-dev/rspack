const { createFsFromVolume, Volume } = require("memfs");

const outputFileSystem = createFsFromVolume(new Volume());

/** @type {import("../../../..").THookCaseConfig} */
module.exports = {
	description: "should call afterProcessAssets correctly",
	options(context) {
		return {
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.compilation.tap("test", compilation => {
							expect(compilation.hooks.afterProcessAssets).toBeTruthy();
							compilation.hooks.afterProcessAssets.tap(
								"should-emit-should-works",
								context.snapped(assets => {
									expect(assets).toBeTruthy();
									expect(assets["main.js"]).toBeTruthy();
								})
							);
						});
					}
				}
			]
		};
	},
	async compiler(context, compiler) {
		compiler.outputFileSystem = outputFileSystem;
	},
	async check() { }
};
