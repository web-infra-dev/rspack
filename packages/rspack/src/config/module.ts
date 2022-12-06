import type {
	RawModuleRuleUse,
	RawModuleRule,
	RawModuleRuleCondition,
	RawModuleOptions
} from "@rspack/binding";
import assert from "assert";
import path from "path";
import { isNil, isPromiseLike } from "../utils";
import { ResolvedContext } from "./context";
import { isUseSourceMap, ResolvedDevtool } from "./devtool";

export interface ModuleRule {
	name?: string;
	test?: string | RegExp;
	resource?: string | RegExp;
	resourceQuery?: string | RegExp;
	use?: ModuleRuleUse[];
	type?: RawModuleRule["type"];
}

export interface Module {
	rules?: ModuleRule[];
	parser?: RawModuleOptions["parser"];
}

interface ResolvedModuleRule {
	test?: RawModuleRule["test"];
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
}

// interface LoaderResult {
// 	content: Buffer | string;
// 	meta: Buffer | string;
// }

interface LoaderResultInternal {
	content: number[];
	sourceMap: number[];
	additionalData: number[];
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

	async function loader(data: Buffer): Promise<Buffer> {
		const payload: LoaderContextInternal = JSON.parse(data.toString("utf-8"));

		let content: string | Buffer = Buffer.from(payload.source);
		let sourceMap: string | SourceMap | undefined = payload.sourceMap;
		let additionalData: AdditionalData | undefined;

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
						content,
						sourceMap,
						additionalData
					});
				}

				const loaderContext: LoaderContext = {
					sourceMap: isUseSourceMap(options.devtool),
					resourcePath: payload.resourcePath,
					resource: payload.resource,
					// Return an empty string if there is no query or fragment
					resourceQuery: payload.resourceQuery || "",
					resourceFragment: payload.resourceFragment || "",
					getOptions() {
						return use.options;
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
					context: path.dirname(payload.resourcePath)
				};

				/**
				 * support loader as string
				 */
				if (typeof use.loader === "string") {
					try {
						let loaderPath = require.resolve(use.loader, {
							paths: [options.context]
						});
						use.loader = require(loaderPath);
					} catch (err) {
						reject(err);
						return;
					}
				}

				let result: Promise<string | Buffer> | string | Buffer | undefined =
					undefined;
				try {
					result = use.loader.apply(loaderContext, [
						use.loader.raw ? Buffer.from(content) : content.toString("utf-8"),
						sourceMap,
						additionalData
					]);
					if (isSync) {
						isDone = true;
						if (result === undefined) {
							resolve({
								content,
								sourceMap,
								additionalData
							});
							return;
						}
						if (isPromiseLike(result)) {
							return result.then(function (result) {
								resolve({
									content: result,
									sourceMap,
									additionalData
								});
							}, reject);
						}
						return resolve({
							content: result,
							sourceMap,
							additionalData
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
						: loaderResult.additionalData) || additionalData;
				content = loaderResult.content || content;
				sourceMap = loaderResult.sourceMap || sourceMap;
			}
		}

		const loaderResultPayload: LoaderResultInternal = {
			content: [...toBuffer(content)],
			sourceMap: !isNil(sourceMap)
				? [
						...toBuffer(
							typeof sourceMap === "string"
								? sourceMap
								: JSON.stringify(sourceMap)
						)
				  ]
				: sourceMap,
			additionalData: !isNil(additionalData)
				? [...toBuffer(JSON.stringify(additionalData))]
				: additionalData
		};

		return Buffer.from(JSON.stringify(loaderResultPayload), "utf-8");
	}

	loader.displayName = `NodeLoaderAdapter(${uses
		.map(item => {
			assert("loader" in item);
			return item.loader.displayName || item.loader.name || "unknown-loader";
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
			loader: Loader;
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
	condition: string | RegExp
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
