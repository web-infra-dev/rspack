import type { Compiler } from "@rspack/core";
import type { THotUpdateContext } from "../../type";

export class TestHotUpdatePlugin {
	constructor(private updateOptions: THotUpdateContext) {}
	apply(compiler: Compiler) {
		let isRebuild = false;
		compiler.hooks.beforeRun.tap("TestHotUpdatePlugin", () => {
			if (isRebuild) {
				compiler.modifiedFiles = new Set(this.updateOptions.changedFiles);
			} else {
				compiler.modifiedFiles = new Set();
			}
			this.updateOptions.changedFiles = [];
			isRebuild = true;
		});

		compiler.hooks.compilation.tap("TestHotUpdatePlugin", compilation => {
			compilation.hooks.additionalTreeRuntimeRequirements.tap(
				"HMR_TEST_PLUGIN",
				(_chunk: any, set: any) => {
					set.add(compiler.webpack.RuntimeGlobals.moduleCache);
				}
			);
			compilation.hooks.runtimeModule.tap(
				"HMR_TEST_PLUGIN",
				(module: any, _set: any) => {
					if (module.constructorName === "DefinePropertyGettersRuntimeModule") {
						module.source.source = Buffer.from(
							`
										__webpack_require__.d = function (exports, definition) {
												for (var key in definition) {
														if (__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
																Object.defineProperty(exports, key, { configurable: true, enumerable: true, get: definition[key] });
														}
												}
										};
										`,
							"utf-8"
						);
					}
				}
			);
		});
	}
}
