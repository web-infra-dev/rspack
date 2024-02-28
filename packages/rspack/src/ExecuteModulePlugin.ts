import { RuntimeGlobals } from ".";
import type { Compiler } from "./Compiler";
import vm from "node:vm";

export default class ExecuteModulePlugin {
	constructor() {}

	apply(compiler: Compiler) {
		compiler.hooks.compilation.tap("executeModule", compilation => {
			compilation.hooks.executeModule.tap(
				"executeModule",
				(options, context) => {
					const moduleObject = options.moduleObject;
					const source = options.codeGenerationResult.get("javascript");
					try {
						const fn = vm.runInThisContext(
							`(function(module, __webpack_module__, __webpack_exports__, exports, ${RuntimeGlobals.require}) {\n${source}\n})`,
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
						let err = e instanceof Error ? e : new Error(e);

						err.stack += printGeneratedCodeForStack(moduleObject.id, source);
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
			 * @param {string[]} lines the lines
			 * @returns {string} the line with line number
			 */
			(line, i, lines) => {
				const iStr = `${i + 1}`;
				return `${" ".repeat(n - iStr.length)}${iStr} | ${line}`;
			}
		)
		.join("\n")}`;
};
