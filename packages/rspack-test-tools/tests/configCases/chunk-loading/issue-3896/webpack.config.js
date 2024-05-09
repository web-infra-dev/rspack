const assert = require("assert");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.thisCompilation.tap(
					"ensure-chunk.groupsIterable-and-group.getParents-work",
					complation => {
						complation.hooks.processAssets.tap(
							"ensure-chunk.groupsIterable-and-group.getParents-work",
							() => {
								assert(complation.chunks.length > 0);
								for (const chunk of complation.chunks) {
									assert(typeof chunk.groupsIterable !== "undefined");
									for (const group of chunk.groupsIterable) {
										assert(typeof group.index === "number");
										assert(Array.isArray(group.getParents()));
										if (group.index === 1) {
											assert(group.name === "main");
										} else {
											assert(typeof group.name === "undefined");
										}
									}
								}
							}
						);
					}
				);
			}
		}
	]
};
