/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/DllReferencePlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { JsBuildMeta } from "@rspack/binding";
import type { CompilationParams } from "../Compilation";
import type { Compiler } from "../Compiler";
import { DllReferenceAgencyPlugin } from "../builtin-plugin";
import { numberOrInfinity } from "../config/utils";
import { z } from "../config/zod";
import { makePathsRelative } from "../util/identifier";
import { memoize } from "../util/memoize";
import { validate } from "../util/validate";
import WebpackError from "./WebpackError";

export type DllReferencePluginOptions =
	| {
			/**
			 * Context of requests in the manifest (or content property) as absolute path.
			 */
			context?: string;
			/**
			 * Extensions used to resolve modules in the dll bundle (only used when using 'scope').
			 */
			extensions?: string[];
			/**
			 * An object containing content and name or a string to the absolute path of the JSON manifest to be loaded upon compilation.
			 */
			manifest: string | DllReferencePluginOptionsManifest;
			/**
			 * The name where the dll is exposed (external name, defaults to manifest.name).
			 */
			name?: string;
			/**
			 * Prefix which is used for accessing the content of the dll.
			 */
			scope?: string;
			/**
			 * How the dll is exposed (libraryTarget, defaults to manifest.type).
			 */
			sourceType?: DllReferencePluginOptionsSourceType;
			/**
			 * The way how the export of the dll bundle is used.
			 */
			type?: "require" | "object";
	  }
	| {
			/**
			 * The mappings from request to module info.
			 */
			content: DllReferencePluginOptionsContent;
			/**
			 * Context of requests in the manifest (or content property) as absolute path.
			 */
			context?: string;
			/**
			 * Extensions used to resolve modules in the dll bundle (only used when using 'scope').
			 */
			extensions?: string[];
			/**
			 * The name where the dll is exposed (external name).
			 */
			name: string;
			/**
			 * Prefix which is used for accessing the content of the dll.
			 */
			scope?: string;
			/**
			 * How the dll is exposed (libraryTarget).
			 */
			sourceType?: DllReferencePluginOptionsSourceType;
			/**
			 * The way how the export of the dll bundle is used.
			 */
			type?: "require" | "object";
	  };
/**
 * The type how the dll is exposed (external type).
 */
export type DllReferencePluginOptionsSourceType =
	| "var"
	| "assign"
	| "this"
	| "window"
	| "global"
	| "commonjs"
	| "commonjs2"
	| "commonjs-module"
	| "amd"
	| "amd-require"
	| "umd"
	| "umd2"
	| "jsonp"
	| "system";

/**
 * An object containing content, name and type.
 */
export interface DllReferencePluginOptionsManifest {
	/**
	 * The mappings from request to module info.
	 */
	content: DllReferencePluginOptionsContent;
	/**
	 * The name where the dll is exposed (external name).
	 */
	name?: string;
	/**
	 * The type how the dll is exposed (external type).
	 */
	type?: DllReferencePluginOptionsSourceType;
}
/**
 * The mappings from request to module info.
 */
export interface DllReferencePluginOptionsContent {
	/**
	 * Module info.
	 */
	[k: string]: {
		/**
		 * Meta information about the module.
		 */
		buildMeta?: JsBuildMeta;
		/**
		 * Information about the provided exports of the module.
		 */
		exports?: string[] | true;
		/**
		 * Module ID.
		 */
		id?: string | number;
	};
}

const getDllReferencePluginOptionsSchema = memoize(() => {
	const dllReferencePluginOptionsContentItem = z
		.object({
			buildMeta: z.custom<JsBuildMeta>(),
			exports: z.array(z.string()).or(z.literal(true)),
			id: z.string().or(numberOrInfinity)
		})
		.partial();

	const dllReferencePluginOptionsContent = z.record(
		z.string(),
		dllReferencePluginOptionsContentItem
	) satisfies z.ZodType<DllReferencePluginOptionsContent>;

	const dllReferencePluginOptionsSourceType = z.enum([
		"var",
		"assign",
		"this",
		"window",
		"global",
		"commonjs",
		"commonjs2",
		"commonjs-module",
		"amd",
		"amd-require",
		"umd",
		"umd2",
		"jsonp",
		"system"
	]) satisfies z.ZodType<DllReferencePluginOptionsSourceType>;

	const dllReferencePluginOptionsManifest = z.object({
		content: dllReferencePluginOptionsContent,
		name: z.string().optional(),
		type: dllReferencePluginOptionsSourceType.optional()
	}) satisfies z.ZodType<DllReferencePluginOptionsManifest>;

	const dllReferencePluginOptions = z.union([
		z.object({
			context: z.string().optional(),
			extensions: z.array(z.string()).optional(),
			manifest: z.string().or(dllReferencePluginOptionsManifest),
			name: z.string().optional(),
			scope: z.string().optional(),
			sourceType: dllReferencePluginOptionsSourceType.optional(),
			type: z.enum(["require", "object"]).optional()
		}),
		z.object({
			content: dllReferencePluginOptionsContent,
			context: z.string().optional(),
			extensions: z.array(z.string()).optional(),
			name: z.string(),
			scope: z.string().optional(),
			sourceType: dllReferencePluginOptionsSourceType.optional(),
			type: z.enum(["require", "object"]).optional()
		})
	]) satisfies z.ZodType<DllReferencePluginOptions>;

	return dllReferencePluginOptions;
});

export class DllReferencePlugin {
	private options: DllReferencePluginOptions;

	private errors: WeakMap<CompilationParams, DllManifestError>;

	constructor(options: DllReferencePluginOptions) {
		validate(options, getDllReferencePluginOptionsSchema);

		this.options = options;
		this.errors = new WeakMap();
	}

	apply(compiler: Compiler) {
		compiler.hooks.beforeCompile.tapPromise(
			DllReferencePlugin.name,
			async params => {
				const manifest = await new Promise<
					DllReferencePluginOptionsManifest | undefined
				>((resolve, reject) => {
					if ("manifest" in this.options) {
						const manifest = this.options.manifest;

						if (typeof manifest === "string") {
							const manifestParameter = manifest;

							compiler.inputFileSystem?.readFile(
								manifestParameter,
								"utf8",
								(err, result) => {
									if (err) return reject(err);

									if (!result)
										return reject(
											new DllManifestError(
												manifestParameter,
												`Can't read anything from ${manifestParameter}`
											)
										);

									try {
										const manifest: DllReferencePluginOptionsManifest =
											JSON.parse(result);
										resolve(manifest);
									} catch (parseError) {
										const manifestPath = makePathsRelative(
											compiler.context,
											manifestParameter,
											compiler.root
										);

										this.errors.set(
											params,
											new DllManifestError(
												manifestPath,
												(parseError as Error).message
											)
										);
									}
								}
							);
						} else {
							resolve(manifest);
						}
					} else {
						resolve(undefined);
					}
				});

				if (!this.errors.has(params)) {
					new DllReferenceAgencyPlugin({
						...this.options,
						type: this.options.type || "require",
						extensions: this.options.extensions || [
							"",
							".js",
							".json",
							".wasm"
						],
						manifest
					}).apply(compiler);
				}
			}
		);

		compiler.hooks.compilation.tap(
			DllReferencePlugin.name,
			(compilation, params) => {
				if (
					"manifest" in this.options &&
					typeof this.options.manifest === "string"
				) {
					const error = this.errors.get(params);
					if (error) {
						compilation.errors.push(error);
					}

					compilation.fileDependencies.add(this.options.manifest);
				}
			}
		);
	}
}

class DllManifestError extends WebpackError {
	constructor(filename: string, message: string) {
		super();

		this.name = "DllManifestError";
		this.message = `Dll manifest ${filename}\n${message}`;
	}
}
