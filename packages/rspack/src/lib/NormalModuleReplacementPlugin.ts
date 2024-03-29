/**
 * Based on [webpack/lib/NormalModuleReplacementPlugin.js]{@link https://github.com/webpack/webpack/blob/29cc4ead7eb6aafc3a5f6d0b10ce41d33d1ad874/lib/NormalModuleReplacementPlugin.js}
 * Licensed with [MIT License]{@link http://www.opensource.org/licenses/mit-license.php}
 * Original Author Tobias Koppers @sokra
 */

import { Compiler } from "../Compiler";
import { ResolveData } from "../Module";
import * as NodePath from "node:path";

type ModuleReplacer = (createData: ResolveData) => void;

export class NormalModuleReplacementPlugin {
	/**
	 * @param {RegExp} resourceRegExp the resource matcher
	 * @param {string|ModuleReplacer} newResource the resource replacement
	 */
	constructor(
		public readonly resourceRegExp: RegExp,
		public readonly newResource: string | ModuleReplacer
	) {}

	apply(compiler: Compiler) {
		const { resourceRegExp, newResource } = this;

		compiler.hooks.normalModuleFactory.tap(
			"NormalModuleReplacementPlugin",
			nmf => {
				nmf.hooks.beforeResolve.tap("NormalModuleReplacementPlugin", result => {
					if (resourceRegExp.test(result.request)) {
						if (typeof newResource === "function") {
							newResource(result);
						} else {
							result.request = newResource;
						}
					}
				});
				nmf.hooks.afterResolve.tap("NormalModuleReplacementPlugin", result => {
					const createData = result.createData || {};
					if (resourceRegExp.test(createData.resource || "")) {
						if (typeof newResource === "function") {
							newResource(result);
						} else {
							if (
								NodePath.posix.isAbsolute(newResource) ||
								NodePath.win32.isAbsolute(newResource)
							) {
								createData.resource = newResource;
							} else {
								createData.resource = NodePath.join(
									NodePath.dirname(createData.resource || ""),
									newResource
								);
							}
						}
					}
				});
			}
		);
	}
}
