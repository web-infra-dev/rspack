const { rspack } = require("@rspack/core");

/** @type {{ value: number, msg: string, info: { builtModules: number, moduleIdentifier?: string } }[]} */
const progressItems = [];

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.ProgressPlugin((value, msg, info) => {
			progressItems.push({ value, msg, info });
		}),
		{
			apply: compiler => {
				compiler.hooks.done.tap("AssertProgressInfoPlugin", () => {
					if (progressItems.length === 0) {
						throw new Error("ProgressPlugin handler should receive updates");
					}

					let lastBuiltModules = -1;
					for (const item of progressItems) {
						if (typeof item.value !== "number") {
							throw new Error("progress value should be a number");
						}
						if (typeof item.msg !== "string") {
							throw new Error("progress msg should be a string");
						}
						if (!item.info || typeof item.info.builtModules !== "number") {
							throw new Error("progress info.builtModules should be a number");
						}
						if (!Number.isInteger(item.info.builtModules)) {
							throw new Error("progress info.builtModules should be an integer");
						}
						if (item.info.builtModules < 0) {
							throw new Error("progress info.builtModules should be >= 0");
						}
						if (item.info.builtModules < lastBuiltModules) {
							throw new Error(
								"progress info.builtModules should be non-decreasing"
							);
						}
						lastBuiltModules = item.info.builtModules;
					}

					const buildModuleItems = progressItems.filter(item =>
						item.msg.startsWith("build modules")
					);
					if (buildModuleItems.length === 0) {
						throw new Error('progress should include "build modules" stage');
					}
					if (!buildModuleItems.some(item => item.info.builtModules > 0)) {
						throw new Error(
							'"build modules" progress should report builtModules > 0 at least once'
						);
					}

					for (const item of progressItems) {
						if (
							item.info.moduleIdentifier !== undefined &&
							typeof item.info.moduleIdentifier !== "string"
						) {
							throw new Error(
								"progress info.moduleIdentifier should be a string when provided"
							);
						}
					}

					const doneItem = progressItems.find(item => item.msg === "done");
					if (!doneItem) {
						throw new Error('progress should include final "done" stage');
					}
					if (doneItem.info.moduleIdentifier !== undefined) {
						throw new Error(
							'final "done" stage should not include moduleIdentifier'
						);
					}
					if (doneItem.info.builtModules <= 0) {
						throw new Error('final "done" stage should report builtModules > 0');
					}
				});
			}
		}
	]
};
