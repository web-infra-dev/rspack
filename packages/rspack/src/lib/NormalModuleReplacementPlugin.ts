/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/NormalModuleReplacementPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import * as NodePath from "node:path";

import type { Compiler } from "../Compiler";
import type { ResolveData } from "../Module";

type ModuleReplacer = (createData: ResolveData) => void;

export class NormalModuleReplacementPlugin {
	/**
	 * Create an instance of the plugin
	 * @param resourceRegExp the resource matcher
	 * @param newResource the resource replacement
	 */
	constructor(
		public readonly resourceRegExp: RegExp,
		public readonly newResource: string | ModuleReplacer
	) {}

	/**
	 * Apply the plugin
	 * @param compiler the compiler instance
	 * @returns
	 */
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
