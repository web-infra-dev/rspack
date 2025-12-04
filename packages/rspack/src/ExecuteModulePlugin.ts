import type { Compiler } from "./Compiler";

export default class ExecuteModulePlugin {
	apply(compiler: Compiler) {
		compiler.hooks.thisCompilation.tap("executeModule", compilation => {
			// remove execution results every new compilation
			const map = compiler.__internal__get_module_execution_results_map();
			map.clear();

			compilation.hooks.executeModule.tap(
				"executeModule",
				(options, context) => {
					const vm = require("node:vm");
					const moduleObject = options.moduleObject;
					const source = options.codeGenerationResult.get("javascript");
					if (source === undefined) return;
					const code = source;

					try {
						const fn = vm.runInThisContext(
							`(function(module, __webpack_module__, __webpack_exports__, exports, ${compiler.rspack.RuntimeGlobals.require}) {\n${code}\n})`,
							{
								filename: moduleObject.id
							}
						);

						fn.call(
							moduleObject.exports,
							moduleObject,
							moduleObject,
							moduleObject.exports,
							moduleObject.exports,
							context.__webpack_require__
						);
					} catch (e: any) {
						const err = e instanceof Error ? e : new Error(e);

						err.stack += printGeneratedCodeForStack(moduleObject.id, code);
						throw err;
					}
				}
			);
		});
	}
}
const printGeneratedCodeForStack = (moduleId: string, code: string) => {
	const lines = code.split("\n");
	const n = `${lines.length}`.length;
	return `\n\nGenerated code for ${moduleId}\n${lines
		.map(
			/**
			 * @param {string} line the line
			 * @param {number} i the index
			 * @returns {string} the line with line number
			 */
			(line, i) => {
				const iStr = `${i + 1}`;
				return `${" ".repeat(n - iStr.length)}${iStr} | ${line}`;
			}
		)
		.join("\n")}`;
};
