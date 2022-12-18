import type {
	RawModuleRuleUse,
	RawModuleRule,
	RawModuleRuleCondition,
	RawModuleOptions,
	JsLoaderContext,
	JsLoaderResult
} from "@rspack/binding";
import assert from "assert";
import path from "path";
import { isNil, isPromiseLike } from "../utils";
import { ResolvedContext } from "./context";
import { isUseSourceMap, ResolvedDevtool } from "./devtool";

export type Condition = string | RegExp;

export interface ModuleRule {
	name?: string;
	test?: Condition;
	include?: Condition | Condition[];
	exclude?: Condition | Condition[];
	resource?: Condition;
	resourceQuery?: Condition;
	use?: ModuleRuleUse[];
	type?: RawModuleRule["type"];
}

export interface Module {
	rules?: ModuleRule[];
	parser?: RawModuleOptions["parser"];
}

interface ResolvedModuleRule {
	test?: RawModuleRule["test"];
	include?: RawModuleRule["include"];
	exclude?: RawModuleRule["exclude"];
	resource?: RawModuleRule["resource"];
	resourceQuery?: RawModuleRule["resourceQuery"];
	use?: RawModuleRuleUse[];
	type?: RawModuleRule["type"];
}

export interface ResolvedModule {
	rules: ResolvedModuleRule[];
	parser?: RawModuleOptions["parser"];
}

interface LoaderContextInternal {
	// TODO: It's not a good way to do this, we should split the `source` into a separate type and avoid using `serde_json`, but it's a temporary solution.
	source: number[];
	sourceMap: string | undefined | null;
	additionalData: AdditionalData | undefined | null;
	resource: string;
	resourcePath: string;
	resourceQuery: string | null;
	resourceFragment: string | null;
	cacheable: boolean;
}
export interface LoaderContext
	extends Pick<
		LoaderContextInternal,
		"resource" | "resourcePath" | "resourceQuery" | "resourceFragment"
	> {
	async(): (
		err: Error | null,
		content: string | Buffer,
		sourceMap?: string | SourceMap,
		additionalData?: AdditionalData
	) => void;
	callback(
		err: Error | null,
		content: string | Buffer,
		sourceMap?: string | SourceMap,
		additionalData?: AdditionalData
	): void;
	cacheable(cacheable: boolean): void;
	sourceMap: boolean;
	rootContext: string;
	context: string;
	getOptions: () => unknown;
}

const toBuffer = (bufLike: string | Buffer): Buffer => {
	if (Buffer.isBuffer(bufLike)) {
		return bufLike;
	} else if (typeof bufLike === "string") {
		return Buffer.from(bufLike);
	}

	throw new Error("Buffer or string expected");
};

export interface ComposeJsUseOptions {
	devtool: ResolvedDevtool;
	context: ResolvedContext;
}

export interface SourceMap {
	version: number;
	sources: string[];
	mappings: string;
	file?: string;
	sourceRoot?: string;
	sourcesContent?: string[];
	names?: string[];
}

export interface AdditionalData {
	[index: string]: any;
	// webpackAST: object;
}

export interface LoaderResult {
	cacheable: boolean;
	content: string | Buffer;
	sourceMap?: string | SourceMap;
	additionalData?: AdditionalData;
}

function composeJsUse(
	uses: ModuleRuleUse[],
	options: ComposeJsUseOptions
): RawModuleRuleUse | null {
	if (!uses.length) {
		return null;
	}

	async function loader(data: JsLoaderContext): Promise<JsLoaderResult> {
		let cacheable: boolean = data.cacheable;
		let content: string | Buffer = data.content;
		let sourceMap: string | SourceMap | undefined =
			data.sourceMap?.toString("utf-8");
		let additionalData: AdditionalData | undefined = data.additionalData
			? JSON.parse(data.additionalData.toString("utf-8"))
			: undefined;

		// Loader is executed from right to left
		for (const use of uses) {
			assert("loader" in use);
			let loaderResult: LoaderResult;

			const p = new Promise<LoaderResult>((resolve, reject) => {
				let isDone = false;
				// Whether a `callback` or `async` is called
				let isSync = true;
				let isError = false; // internal error
				let reportedError = false;

				function callback(
					err: Error | null,
					content: string | Buffer,
					sourceMap?: string | SourceMap,
					additionalData?: AdditionalData
				) {
					if (isDone) {
						if (reportedError) return; // ignore
						err = new Error("callback(): The callback was already called.");
					}
					isSync = false;
					isDone = true;

					if (err) {
						isError = true;
						reject(err);
						return;
					}

					resolve({
						cacheable,
						content,
						sourceMap,
						additionalData
					});
				}

				const loaderContext: LoaderContext = {
					sourceMap: isUseSourceMap(options.devtool),
					resourcePath: data.resourcePath,
					resource: data.resource,
					// Return an empty string if there is no query or fragment
					resourceQuery: data.resourceQuery || "",
					resourceFragment: data.resourceFragment || "",
					getOptions() {
						return use.options;
					},
					cacheable(value) {
						cacheable = value;
					},
					async() {
						if (isDone) {
							if (reportedError) return; // ignore
							reject(new Error("async(): The callback was already called."));
						}
						isSync = false;
						return callback;
					},
					callback,
					rootContext: options.context,
					context: path.dirname(data.resourcePath)
				};

				/**
				 * support loader as string
				 */
				let loader: Loader | undefined;
				if (typeof use.loader === "string") {
					try {
						let loaderPath = require.resolve(use.loader, {
							paths: [options.context]
						});
						loader = require(loaderPath);
					} catch (err) {
						reject(err);
						return;
					}
				} else {
					loader = use.loader;
				}

				let result: Promise<string | Buffer> | string | Buffer | undefined =
					undefined;
				try {
					result = loader.apply(loaderContext, [
						loader.raw ? Buffer.from(content) : content.toString("utf-8"),
						sourceMap,
						additionalData
					]);
					if (isSync) {
						isDone = true;
						if (result === undefined) {
							resolve({
								content,
								sourceMap,
								additionalData,
								cacheable
							});
							return;
						}
						if (isPromiseLike(result)) {
							return result.then(function (result) {
								resolve({
									content: result,
									sourceMap,
									additionalData,
									cacheable
								});
							}, reject);
						}
						return resolve({
							content: result,
							sourceMap,
							additionalData,
							cacheable
						});
					}
				} catch (err) {
					if (isError) {
						reject(err);
						return;
					}
					if (isDone) {
						// loader is already "done", so we cannot use the callback function
						// for better debugging we print the error on the console
						if (typeof err === "object" && err.stack) console.error(err.stack);
						else console.error(err);
						reject(err);
						return;
					}
					isDone = true;
					reportedError = true;
					reject(err);
				}
			});

			if ((loaderResult = await p)) {
				additionalData =
					(typeof loaderResult.additionalData === "string"
						? JSON.parse(loaderResult.additionalData)
						: loaderResult.additionalData) ?? additionalData;
				content = loaderResult.content ?? content;
				sourceMap = loaderResult.sourceMap ?? sourceMap;
				cacheable = loaderResult.cacheable ?? cacheable;
			}
		}

		return {
			cacheable: cacheable,
			content: toBuffer(content),
			sourceMap: sourceMap
				? toBuffer(
						typeof sourceMap === "string"
							? sourceMap
							: JSON.stringify(sourceMap)
				  )
				: undefined,
			additionalData: additionalData
				? toBuffer(JSON.stringify(additionalData))
				: undefined
		};
	}

	loader.displayName = `NodeLoaderAdapter(${uses
		.map(item => {
			assert("loader" in item);
			let loader: Loader | null;
			if (typeof item.loader === "string") {
				try {
					const path = require.resolve(item.loader, {
						paths: [options.context]
					});
					loader = require(path);
				} catch (e) {
					loader = null;
				}
			} else {
				loader = item.loader;
			}
			return loader?.displayName || loader?.name || "unknown-loader";
		})
		.join(" -> ")})`;
	return {
		loader
	};
}

export interface Loader {
	(
		this: LoaderContext,
		content: string | Buffer,
		sourceMap?: string | SourceMap,
		additionalData?: AdditionalData
	): void;
	displayName?: string;
	raw?: boolean;
}

type BuiltinLoader = string;

type ModuleRuleUse =
	| {
			builtinLoader: BuiltinLoader;
			options?: unknown;
			name?: string;
	  }
	| {
			// String represents a path to the loader
			loader: Loader | string;
			options?: unknown;
			name?: string;
	  };

export function createRawModuleRuleUses(
	uses: ModuleRuleUse[],
	options: ComposeJsUseOptions
): RawModuleRuleUse[] {
	return createRawModuleRuleUsesImpl([...uses].reverse(), options);
}

function createRawModuleRuleUsesImpl(
	uses: ModuleRuleUse[],
	options: ComposeJsUseOptions
): RawModuleRuleUse[] {
	if (!uses.length) {
		return [];
	}
	const index = uses.findIndex(use => "builtinLoader" in use);
	if (index < 0) {
		return [composeJsUse(uses, options)];
	}

	const before = uses.slice(0, index);
	const after = uses.slice(index + 1);
	return [
		composeJsUse(before, options),
		createNativeUse(uses[index]),
		...createRawModuleRuleUsesImpl(after, options)
	].filter((item): item is RawModuleRuleUse => Boolean(item));
}

function createNativeUse(use: ModuleRuleUse): RawModuleRuleUse {
	assert("builtinLoader" in use);

	if (use.builtinLoader === "sass-loader") {
		(use.options ??= {} as any).__exePath = require.resolve(
			`@tmp-sass-embedded/${process.platform}-${
				process.arch
			}/dart-sass-embedded/dart-sass-embedded${
				process.platform === "win32" ? ".bat" : ""
			}`
		);
	}

	return {
		builtinLoader: use.builtinLoader,
		options: JSON.stringify(use.options)
	};
}

function resolveModuleRuleCondition(
	condition: Condition
): RawModuleRuleCondition {
	if (typeof condition === "string") {
		return {
			type: "string",
			matcher: condition
		};
	}

	if (condition instanceof RegExp) {
		return {
			type: "regexp",
			matcher: condition.source
		};
	}

	throw new Error(
		`Unsupported condition type ${typeof condition}, value: ${condition}`
	);
}

function resolveModuleRuleConditions(
	conditions: Condition[]
): RawModuleRuleCondition[] {
	return conditions.map(resolveModuleRuleCondition);
}

export function resolveModuleOptions(
	module: Module = {},
	options: ComposeJsUseOptions
): ResolvedModule {
	const rules = (module.rules ?? []).map(rule => {
		// FIXME: use error handler instead of throwing
		if ((rule as any)?.loader) {
			throw new Error("`Rule.loader` is not supported, use `Rule.use` instead");
		}

		if ((rule as any)?.uses) {
			throw new Error(
				"`Rule.uses` is deprecated for aligning with webpack, use `Rule.use` instead"
			);
		}

		return {
			...rule,
			test: isNil(rule.test) ? null : resolveModuleRuleCondition(rule.test),
			include: isNil(rule.include)
				? null
				: Array.isArray(rule.include)
				? resolveModuleRuleConditions(rule.include)
				: [resolveModuleRuleCondition(rule.include)],
			exclude: isNil(rule.exclude)
				? null
				: Array.isArray(rule.exclude)
				? resolveModuleRuleConditions(rule.exclude)
				: [resolveModuleRuleCondition(rule.exclude)],
			resource: isNil(rule.resource)
				? null
				: resolveModuleRuleCondition(rule.resource),
			resourceQuery: isNil(rule.resourceQuery)
				? null
				: resolveModuleRuleCondition(rule.resourceQuery),
			use: createRawModuleRuleUses(rule.use || [], options)
		};
	});
	return {
		parser: module.parser,
		rules
	};
}
