const assert = require("assert");
const { createFsFromVolume, Volume } = require("memfs");

const outputFileSystem = createFsFromVolume(new Volume());

let hasMainJs = false;

/** @type {import("../../../..").THookCaseConfig} */
module.exports = {
	description: "should work with assetEmitted",
	options(context) {
		return {
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.assetEmitted.tap("plugin", (filename, info) => {
							return context.snapped(async (filename, info) => {
								if (filename === "main.js") {
									assert(info.targetPath.includes("main.js"));
									hasMainJs = true;
								}
							})(filename, info);
						});
					}
				}
			]
		};
	},
	async compiler(context, compiler) {
		compiler.outputFileSystem = outputFileSystem;
	},
	async check() {
		expect(hasMainJs).toBeTruthy();
	}
};
