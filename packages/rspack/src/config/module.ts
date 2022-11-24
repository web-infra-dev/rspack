import type {
	RawModuleRuleUse,
	RawModuleRule,
	RawModuleRuleCondition,
	RawModuleOptions
} from "@rspack/binding";
import assert from "node:assert";
import path from "node:path";
import { isNil } from "../utils";
import { ResolvedContext } from "./context";
import { isUseSourceMap, ResolvedDevtool } from "./devtool";

export interface ModuleRule {
	name?: string;
	test?: string | RegExp;
	resource?: string | RegExp;
	resourceQuery?: string | RegExp;
	uses?: ModuleRuleUse[];
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
	uses?: RawModuleRuleUse[];
	type?: RawModuleRule["type"];
}

export interface ResolvedModule {
	rules: ResolvedModuleRule[];
	parser?: RawModuleOptions["parser"];
}

interface LoaderContextInternal {
	// TODO: It's not a good way to do this, we should split the `source` into a separate type and avoid using `serde_json`, but it's a temporary solution.
	source: number[];
	sourceMap: string | null;
	resource: string;
	resourcePath: string;
	resourceQuery: string | null;
	resourceFragment: string | null;
}

interface LoaderResult {
	content: Buffer | string;
	meta: Buffer | string;
}

interface LoaderThreadsafeResult {
	id: number;
	p: LoaderResultInternal | null | undefined;
}

interface LoaderResultInternal {
	content: number[];
	meta: number[];
}

export interface LoaderContext
	extends Pick<
		LoaderContextInternal,
		| "resource"
		| "resourcePath"
		| "resourceQuery"
		| "resourceFragment"
		| "sourceMap"
	> {
	source: {
		getCode(): string;
		getBuffer(): Buffer;
	};
	useSourceMap: boolean;
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

interface LoaderThreadsafeContext {
	id: number;
	p: LoaderContextInternal;
}

export interface ComposeJsUseOptions {
	devtool: ResolvedDevtool;
	context: ResolvedContext;
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

		const loaderContextInternal: LoaderContextInternal = {
			source: payload.source,
			sourceMap: payload.sourceMap,
			resourcePath: payload.resourcePath,
			resourceQuery: payload.resourceQuery,
			resource: payload.resource,
			resourceFragment: payload.resourceFragment
		};

		let sourceBuffer = Buffer.from(loaderContextInternal.source);
		let meta = Buffer.from("");
		// Loader is executed from right to left
		for (const use of uses) {
			assert("loader" in use);
			/**
			 * support loader as string
			 */
			if (typeof use.loader === "string") {
				let loaderPath = require.resolve(use.loader, {
					paths: [options.context]
				});
				use.loader = require(loaderPath);
			}
			const loaderContext = {
				...loaderContextInternal,
				source: {
					getCode(): string {
						return sourceBuffer.toString("utf-8");
					},
					getBuffer(): Buffer {
						return sourceBuffer;
					}
				},
				getOptions() {
					return use.options;
				},
				useSourceMap: isUseSourceMap(options.devtool),
				rootContext: options.context,
				context: path.dirname(loaderContextInternal.resourcePath)
			};
			let loaderResult: LoaderResult;
			if (
				(loaderResult = await Promise.resolve().then(() =>
					use.loader.apply(loaderContext, [loaderContext])
				))
			) {
				const content = loaderResult.content;
				meta = meta.length > 0 ? meta : toBuffer(loaderResult.meta);
				sourceBuffer = toBuffer(content);
			}
		}

		const loaderResultPayload: LoaderResultInternal = {
			content: [...sourceBuffer],
			meta: [...meta]
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

interface JsLoader {
	(this: LoaderContext, loaderContext: LoaderContext):
		| Promise<LoaderResult | void>
		| LoaderResult
		| void;
	displayName?: string;
}

type BuiltinLoader = string;

type ModuleRuleUse =
	| {
			builtinLoader: BuiltinLoader;
			options?: unknown;
			name?: string;
	  }
	| {
			loader: JsLoader;
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
	const rules = (module.rules ?? []).map(rule => ({
		...rule,
		test: isNil(rule.test) ? null : resolveModuleRuleCondition(rule.test),
		resource: isNil(rule.resource)
			? null
			: resolveModuleRuleCondition(rule.resource),
		resourceQuery: isNil(rule.resourceQuery)
			? null
			: resolveModuleRuleCondition(rule.resourceQuery),
		uses: createRawModuleRuleUses(rule.uses || [], options)
	}));
	return {
		parser: module.parser,
		rules
	};
}
