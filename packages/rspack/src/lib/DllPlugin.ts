/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/DllPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { Compiler } from "../Compiler";
import { LibManifestPlugin } from "../builtin-plugin";
import { DllEntryPlugin } from "../builtin-plugin/DllEntryPlugin";
import { FlagAllModulesAsUsedPlugin } from "../builtin-plugin/FlagAllModulesAsUsedPlugin";
import { z } from "../config/zod";
import { memoize } from "../util/memoize";
import { validate } from "../util/validate";

export type DllPluginOptions = {
	/**
	 * Context of requests in the manifest file (defaults to the webpack context).
	 */
	context?: string;

	/**
	 * If true, only entry points will be exposed.
	 * @default true
	 */
	entryOnly?: boolean;

	/**
	 * If true, manifest json file (output) will be formatted.
	 */
	format?: boolean;

	/**
	 * Name of the exposed dll function (external name, use value of 'output.library').
	 */
	name?: string;

	/**
	 * Absolute path to the manifest json file (output).
	 */
	path: string;

	/**
	 * Type of the dll bundle (external type, use value of 'output.libraryTarget').
	 */
	type?: string;
};

const getDllPluginOptionsSchema = memoize(
	() =>
		z.object({
			context: z.string().optional(),
			entryOnly: z.boolean().optional(),
			format: z.boolean().optional(),
			name: z.string().optional(),
			path: z.string(),
			type: z.string().optional()
		}) satisfies z.ZodType<DllPluginOptions>
);

export class DllPlugin {
	private options: DllPluginOptions;

	constructor(options: DllPluginOptions) {
		validate(options, getDllPluginOptionsSchema);
		this.options = {
			...options,
			entryOnly: options.entryOnly !== false
		};
	}

	apply(compiler: Compiler) {
		compiler.hooks.entryOption.tap(DllPlugin.name, (context, entry) => {
			if (typeof entry === "function") {
				throw new Error(
					"DllPlugin doesn't support dynamic entry (function) yet"
				);
			}

			for (const name of Object.keys(entry)) {
				const options = {
					name
				};
				const entries = entry[name].import || [];

				new DllEntryPlugin(context, entries, options).apply(compiler);
			}

			return true;
		});

		new LibManifestPlugin(this.options).apply(compiler);

		if (!this.options.entryOnly) {
			new FlagAllModulesAsUsedPlugin("DllPlugin").apply(compiler);
		}
	}
}
