const assert = require("assert");

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
	async check() {
		expect(hasMainJs).toBeTruthy();
	}
};
