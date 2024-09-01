/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/ContextReplacementPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { Compiler } from "../Compiler";
import type {
	ContextModuleFactoryAfterResolveResult,
	ContextModuleFactoryBeforeResolveResult
} from "../Module";
import { join } from "../util/fs";

type ModuleReplacer = (
	createData:
		| ContextModuleFactoryBeforeResolveResult
		| ContextModuleFactoryAfterResolveResult
) => void;

export class ContextReplacementPlugin {
	/**
	 * @param resourceRegExp A regular expression that determines which files will be selected
	 * @param newContentResource A new resource to replace the match
	 */
	constructor(
		public readonly resourceRegExp: RegExp,
		public readonly newContentResource: string | ModuleReplacer
	) {}

	/**
	 * Apply the plugin
	 * @param compiler the compiler instance
	 * @returns
	 */
	apply(compiler: Compiler) {
		const resourceRegExp = this.resourceRegExp;
		const newContentResource = this.newContentResource;

		compiler.hooks.contextModuleFactory.tap("ContextReplacementPlugin", cmf => {
			cmf.hooks.beforeResolve.tap("ContextReplacementPlugin", result => {
				if (
					result !== false &&
					result.request &&
					resourceRegExp.test(result.request)
				) {
					if (typeof newContentResource === "function") {
						newContentResource(result);
					} else {
						result.request = newContentResource;
					}
				}
				return result;
			});
			cmf.hooks.afterResolve.tap("ContextReplacementPlugin", result => {
				if (result !== false && resourceRegExp.test(result.resource)) {
					if (typeof newContentResource === "function") {
						const origResource = result.resource;
						newContentResource(result);
						if (
							result.resource !== origResource &&
							!result.resource.startsWith("/") &&
							(result.resource.length <= 1 || result.resource[1] !== ":")
						) {
							// When the function changed it to an relative path
							result.resource = join(
								compiler.inputFileSystem,
								origResource,
								result.resource
							);
						}
					} else {
						if (
							newContentResource.startsWith("/") ||
							(newContentResource.length > 1 && newContentResource[1] === ":")
						) {
							result.resource = newContentResource;
						} else {
							result.resource = join(
								compiler.inputFileSystem,
								result.resource,
								newContentResource
							);
						}
					}
				}
				return result;
			});
		});
	}
}
