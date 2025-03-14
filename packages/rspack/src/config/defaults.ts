/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/config/defaults.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import assert from "node:assert";
import fs from "node:fs";
import path from "node:path";

import { ASSET_MODULE_TYPE, JSON_MODULE_TYPE } from "../ModuleTypeConstants";
import { Template } from "../Template";
import {
	LightningCssMinimizerRspackPlugin,
	SwcJsMinimizerRspackPlugin
} from "../builtin-plugin";
import { isNil } from "../util";
import { assertNotNill } from "../util/assertNotNil";
import { cleverMerge } from "../util/cleverMerge";
import type {
	EntryDescriptionNormalized,
	EntryNormalized,
	ExperimentsNormalized,
	OutputNormalized,
	RspackOptionsNormalized
} from "./normalization";
import {
	getDefaultTarget,
	getTargetProperties,
	getTargetsProperties
} from "./target";
import type {
	Context,
	ExternalsPresets,
	InfrastructureLogging,
	JavascriptParserOptions,
	JsonGeneratorOptions,
	Library,
	Loader,
	Mode,
	ModuleOptions,
	Node,
	Optimization,
	Performance,
	ResolveOptions,
	RspackFutureOptions,
	RuleSetRules,
	SnapshotOptions
} from "./types";

export const applyRspackOptionsDefaults = (
	options: RspackOptionsNormalized
) => {
	F(options, "context", () => process.cwd());
	F(options, "target", () => {
		return getDefaultTarget(options.context!);
	});

	const { mode, target } = options;
	assert(!isNil(target));

	const targetProperties =
		target === false
			? (false as const)
			: typeof target === "string"
				? getTargetProperties(target, options.context!)
				: getTargetsProperties(target, options.context!);

	const development = mode === "development";
	const production = mode === "production" || !mode;

	if (typeof options.entry !== "function") {
		for (const key of Object.keys(options.entry)) {
			F(options.entry[key], "import", () => ["./src"]);
		}
	}

	F(options, "devtool", () => (development ? "eval" : false));
	D(options, "watch", false);
	D(options, "profile", false);
	// IGNORE(bail): bail is default to false in webpack, but it's set in `Compilation`
	D(options, "bail", false);

	// IGNORE(cache): cache is default to { type: "memory" } in webpack when the mode is development,
	// but Rspack currently does not support this option
	F(options, "cache", () => development);

	applyExperimentsDefaults(options.experiments, {
		production,
		development
	});

	if (options.cache === false) {
		options.experiments.cache = false;
	}

	applySnapshotDefaults(options.snapshot, { production });

	applyModuleDefaults(options.module, {
		asyncWebAssembly: options.experiments.asyncWebAssembly!,
		css: options.experiments.css,
		targetProperties,
		mode: options.mode
	});

	applyOutputDefaults(options.output, {
		context: options.context!,
		targetProperties,
		isAffectedByBrowserslist:
			target === undefined ||
			(typeof target === "string" && target.startsWith("browserslist")) ||
			(Array.isArray(target) &&
				target.some(target => target.startsWith("browserslist"))),
		outputModule: options.experiments.outputModule,
		development,
		entry: options.entry,
		futureDefaults: options.experiments.futureDefaults!
	});
	// bundlerInfo is affected by outputDefaults so must be executed after outputDefaults
	applybundlerInfoDefaults(
		options.experiments.rspackFuture,
		options.output.library
	);

	applyExternalsPresetsDefaults(options.externalsPresets, {
		targetProperties
	});

	F(options, "externalsType", () => {
		return options.output.library
			? // loose type 'string', actual type is "commonjs" | "var" | "commonjs2"....
				(options.output.library.type as any)
			: options.output.module
				? "module-import"
				: "var";
	});

	applyNodeDefaults(options.node, {
		targetProperties,
		outputModule: options.output.module
	});

	applyLoaderDefaults(options.loader, {
		targetProperties,
		environment: options.output.environment
	});

	F(options, "performance", () =>
		production &&
		targetProperties &&
		(targetProperties.browser || targetProperties.browser === null)
			? {}
			: false
	);
	applyPerformanceDefaults(options.performance!, {
		production
	});

	applyOptimizationDefaults(options.optimization, {
		production,
		development,
		css: options.experiments.css!
	});

	options.resolve = cleverMerge(
		getResolveDefaults({
			context: options.context!,
			targetProperties,
			mode: options.mode,
			css: options.experiments.css!
		}),
		options.resolve
	);

	options.resolveLoader = cleverMerge(
		getResolveLoaderDefaults(),
		options.resolveLoader
	);
};

export const applyRspackOptionsBaseDefaults = (
	options: RspackOptionsNormalized
) => {
	F(options, "context", () => process.cwd());
	applyInfrastructureLoggingDefaults(options.infrastructureLogging);
};

const applyInfrastructureLoggingDefaults = (
	infrastructureLogging: InfrastructureLogging
) => {
	F(infrastructureLogging, "stream", () => process.stderr);
	const tty =
		(infrastructureLogging as any).stream.isTTY && process.env.TERM !== "dumb";
	D(infrastructureLogging, "level", "info");
	D(infrastructureLogging, "debug", false);
	D(infrastructureLogging, "colors", tty);
	D(infrastructureLogging, "appendOnly", !tty);
};

const applyExperimentsDefaults = (
	experiments: ExperimentsNormalized,
	{ production, development }: { production: boolean; development: boolean }
) => {
	// IGNORE(experiments.cache): In webpack, cache is undefined by default
	F(experiments, "cache", () => development);

	D(experiments, "futureDefaults", false);
	// IGNORE(experiments.lazyCompilation): In webpack, lazyCompilation is undefined by default
	D(experiments, "lazyCompilation", false);
	D(experiments, "asyncWebAssembly", experiments.futureDefaults);
	D(experiments, "css", experiments.futureDefaults ? true : undefined);
	D(experiments, "layers", false);
	D(experiments, "topLevelAwait", true);

	// IGNORE(experiments.incremental): Rspack specific configuration for incremental
	D(experiments, "incremental", !production ? {} : false);
	if (typeof experiments.incremental === "object") {
		D(experiments.incremental, "make", true);
		D(experiments.incremental, "inferAsyncModules", false);
		D(experiments.incremental, "providedExports", false);
		D(experiments.incremental, "dependenciesDiagnostics", false);
		D(experiments.incremental, "sideEffects", false);
		D(experiments.incremental, "buildChunkGraph", false);
		D(experiments.incremental, "moduleIds", false);
		D(experiments.incremental, "chunkIds", false);
		D(experiments.incremental, "modulesHashes", false);
		D(experiments.incremental, "modulesCodegen", false);
		D(experiments.incremental, "modulesRuntimeRequirements", false);
		D(experiments.incremental, "chunksRuntimeRequirements", false);
		D(experiments.incremental, "chunksHashes", false);
		D(experiments.incremental, "chunksRender", false);
		D(experiments.incremental, "emitAssets", true);
	}
	// IGNORE(experiments.rspackFuture): Rspack specific configuration
	D(experiments, "rspackFuture", {});
	// rspackFuture.bundlerInfo default value is applied after applyDefaults

	// IGNORE(experiments.parallelCodeSplitting): Rspack specific configuration for new code splitting algorithm
	D(experiments, "parallelCodeSplitting", false);
};

const applybundlerInfoDefaults = (
	rspackFuture?: RspackFutureOptions,
	library?: Library
) => {
	if (typeof rspackFuture === "object") {
		D(rspackFuture, "bundlerInfo", {});
		if (typeof rspackFuture.bundlerInfo === "object") {
			D(
				rspackFuture.bundlerInfo,
				"version",
				require("../../package.json").version
			);
			D(rspackFuture.bundlerInfo, "bundler", "rspack");
			// don't inject for library mode
			D(rspackFuture.bundlerInfo, "force", !library);
		}
	}
};

const applySnapshotDefaults = (
	_snapshot: SnapshotOptions,
	_env: { production: boolean }
) => {};

const applyJavascriptParserOptionsDefaults = (
	parserOptions: JavascriptParserOptions
) => {
	D(parserOptions, "dynamicImportMode", "lazy");
	D(parserOptions, "dynamicImportPrefetch", false);
	D(parserOptions, "dynamicImportPreload", false);
	D(parserOptions, "url", true);
	D(parserOptions, "exprContextCritical", true);
	D(parserOptions, "wrappedContextCritical", false);
	D(parserOptions, "wrappedContextRegExp", /.*/);
	D(parserOptions, "strictExportPresence", false);
	D(parserOptions, "requireAsExpression", true);
	D(parserOptions, "requireDynamic", true);
	D(parserOptions, "requireResolve", true);
	D(parserOptions, "importDynamic", true);
	D(parserOptions, "worker", ["..."]);
	D(parserOptions, "importMeta", true);
};

const applyJsonGeneratorOptionsDefaults = (
	generatorOptions: JsonGeneratorOptions
) => {
	D(generatorOptions, "JSONParse", true);
};

const applyModuleDefaults = (
	module: ModuleOptions,
	{
		asyncWebAssembly,
		css,
		targetProperties,
		mode
	}: {
		asyncWebAssembly: boolean;
		css?: boolean;
		targetProperties: any;
		mode?: Mode;
	}
) => {
	assertNotNill(module.parser);
	assertNotNill(module.generator);

	// IGNORE(module.parser): already check to align in 2024.6.27
	F(module.parser, ASSET_MODULE_TYPE, () => ({}));
	assertNotNill(module.parser.asset);
	F(module.parser.asset, "dataUrlCondition", () => ({}));
	if (typeof module.parser.asset.dataUrlCondition === "object") {
		D(module.parser.asset.dataUrlCondition, "maxSize", 8096);
	}

	F(module.parser, "javascript", () => ({}));
	assertNotNill(module.parser.javascript);
	applyJavascriptParserOptionsDefaults(module.parser.javascript);

	F(module.parser, JSON_MODULE_TYPE, () => ({}));
	assertNotNill(module.parser[JSON_MODULE_TYPE]);
	D(
		module.parser[JSON_MODULE_TYPE],
		"exportsDepth",
		mode === "development" ? 1 : Number.MAX_SAFE_INTEGER
	);

	F(module.generator, "json", () => ({}));
	assertNotNill(module.generator.json);
	applyJsonGeneratorOptionsDefaults(module.generator.json);

	if (css) {
		F(module.parser, "css", () => ({}));
		assertNotNill(module.parser.css);
		D(module.parser.css, "namedExports", true);

		F(module.parser, "css/auto", () => ({}));
		assertNotNill(module.parser["css/auto"]);
		D(module.parser["css/auto"], "namedExports", true);

		F(module.parser, "css/module", () => ({}));
		assertNotNill(module.parser["css/module"]);
		D(module.parser["css/module"], "namedExports", true);

		// IGNORE(module.generator): already check to align in 2024.6.27
		F(module.generator, "css", () => ({}));
		assertNotNill(module.generator.css);
		D(
			module.generator.css,
			"exportsOnly",
			!targetProperties || !targetProperties.document
		);
		D(module.generator.css, "esModule", true);

		F(module.generator, "css/auto", () => ({}));
		assertNotNill(module.generator["css/auto"]);
		D(
			module.generator["css/auto"],
			"exportsOnly",
			!targetProperties || !targetProperties.document
		);
		D(module.generator["css/auto"], "exportsConvention", "as-is");
		D(
			module.generator["css/auto"],
			"localIdentName",
			"[uniqueName]-[id]-[local]"
		);
		D(module.generator["css/auto"], "esModule", true);

		F(module.generator, "css/module", () => ({}));
		assertNotNill(module.generator["css/module"]);
		D(
			module.generator["css/module"],
			"exportsOnly",
			!targetProperties || !targetProperties.document
		);
		D(module.generator["css/module"], "exportsConvention", "as-is");
		D(
			module.generator["css/module"],
			"localIdentName",
			"[uniqueName]-[id]-[local]"
		);
		D(module.generator["css/module"], "esModule", true);
	}

	// IGNORE(module.defaultRules): Rspack does not support `rule.assert`
	// https://github.com/webpack/webpack/blob/main/lib/config/defaults.js#L839
	A(module, "defaultRules", () => {
		const esm = {
			type: "javascript/esm",
			resolve: {
				byDependency: {
					esm: {
						fullySpecified: true
					}
				}
			}
		};
		const commonjs = {
			type: "javascript/dynamic"
		};
		const rules: RuleSetRules = [
			{
				mimetype: "application/node",
				type: "javascript/auto"
			},
			{
				test: /\.json$/i,
				type: "json"
			},
			{
				mimetype: "application/json",
				type: "json"
			},
			{
				test: /\.mjs$/i,
				...esm
			},
			{
				test: /\.js$/i,
				descriptionData: {
					type: "module"
				},
				...esm
			},
			{
				test: /\.cjs$/i,
				...commonjs
			},
			{
				test: /\.js$/i,
				descriptionData: {
					type: "commonjs"
				},
				...commonjs
			},
			{
				mimetype: {
					or: ["text/javascript", "application/javascript"]
				},
				...esm
			}
		];

		if (asyncWebAssembly) {
			const wasm = {
				type: "webassembly/async",
				rules: [
					{
						descriptionData: {
							type: "module"
						},
						resolve: {
							fullySpecified: true
						}
					}
				]
			};
			rules.push({
				test: /\.wasm$/i,
				...wasm
			});
			rules.push({
				mimetype: "application/wasm",
				...wasm
			});
		}

		if (css) {
			const resolve = {
				fullySpecified: true,
				preferRelative: true
			};
			rules.push({
				test: /\.css$/i,
				type: "css/auto",
				resolve
			});
			rules.push({
				mimetype: "text/css+module",
				type: "css/module",
				resolve
			});
			rules.push({
				mimetype: "text/css",
				type: "css",
				resolve
			});
		}

		rules.push(
			{
				dependency: "url",
				oneOf: [
					{
						scheme: /^data$/,
						type: "asset/inline"
					},
					{
						type: "asset/resource"
					}
				]
			},
			{
				with: { type: "json" },
				type: "json"
			}
		);

		return rules;
	});
};

const applyOutputDefaults = (
	output: OutputNormalized,
	{
		context,
		outputModule,
		targetProperties: tp,
		isAffectedByBrowserslist,
		development,
		entry,
		futureDefaults
	}: {
		context: Context;
		outputModule?: boolean;
		targetProperties: any;
		isAffectedByBrowserslist: boolean;
		development: boolean;
		entry: EntryNormalized;
		futureDefaults: boolean;
	}
) => {
	const getLibraryName = (library: Library): string => {
		const libraryName =
			typeof library === "object" &&
			library &&
			!Array.isArray(library) &&
			"type" in library
				? library.name
				: library;
		if (Array.isArray(libraryName)) {
			return libraryName.join(".");
		}
		if (typeof libraryName === "object") {
			return getLibraryName(libraryName.root);
		}
		if (typeof libraryName === "string") {
			return libraryName;
		}
		return "";
	};
	F(output, "uniqueName", () => {
		const libraryName = getLibraryName(output.library).replace(
			/^\[(\\*[\w:]+\\*)\](\.)|(\.)\[(\\*[\w:]+\\*)\](?=\.|$)|\[(\\*[\w:]+\\*)\]/g,
			(m, a, d1, d2, b, c) => {
				const content = a || b || c;
				return content.startsWith("\\") && content.endsWith("\\")
					? `${d2 || ""}[${content.slice(1, -1)}]${d1 || ""}`
					: "";
			}
		);
		if (libraryName) return libraryName;
		const pkgPath = path.resolve(context, "package.json");
		try {
			const packageInfo = JSON.parse(fs.readFileSync(pkgPath, "utf-8"));
			return packageInfo.name || "";
		} catch (err) {
			const e = err as Error & { code: string };
			if (e.code !== "ENOENT") {
				e.message += `\nwhile determining default 'output.uniqueName' from 'name' in ${pkgPath}`;
				throw e;
			}
			return "";
		}
	});
	F(output, "devtoolNamespace", () => output.uniqueName);
	F(output, "module", () => !!outputModule);
	D(output, "filename", output.module ? "[name].mjs" : "[name].js");
	F(output, "iife", () => !output.module);

	F(output, "chunkFilename", () => {
		const filename = output.filename!;
		if (typeof filename !== "function") {
			const hasName = filename.includes("[name]");
			const hasId = filename.includes("[id]");
			const hasChunkHash = filename.includes("[chunkhash]");
			const hasContentHash = filename.includes("[contenthash]");
			// Anything changing depending on chunk is fine
			if (hasChunkHash || hasContentHash || hasName || hasId) return filename;
			// Otherwise prefix "[id]." in front of the basename to make it changing
			return filename.replace(/(^|\/)([^/]*(?:\?|$))/, "$1[id].$2");
		}
		return "[id].js";
	});
	F(output, "cssFilename", () => {
		const filename = output.filename!;
		if (typeof filename !== "function") {
			return filename.replace(/\.[mc]?js(\?|$)/, ".css$1");
		}
		return "[id].css";
	});
	F(output, "cssChunkFilename", () => {
		const chunkFilename = output.chunkFilename!;
		if (typeof chunkFilename !== "function") {
			return chunkFilename.replace(/\.[mc]?js(\?|$)/, ".css$1");
		}
		return "[id].css";
	});
	D(
		output,
		"hotUpdateChunkFilename",
		`[id].[fullhash].hot-update.${output.module ? "mjs" : "js"}`
	);
	D(output, "hotUpdateMainFilename", "[runtime].[fullhash].hot-update.json");

	const uniqueNameId = Template.toIdentifier(output.uniqueName);
	F(output, "hotUpdateGlobal", () => `webpackHotUpdate${uniqueNameId}`);
	F(output, "chunkLoadingGlobal", () => `webpackChunk${uniqueNameId}`);
	D(output, "assetModuleFilename", "[hash][ext][query]");
	D(output, "webassemblyModuleFilename", "[hash].module.wasm");
	D(output, "compareBeforeEmit", true);
	F(output, "path", () => path.join(process.cwd(), "dist"));
	F(output, "pathinfo", () => development);
	D(
		output,
		"publicPath",
		tp && (tp.document || tp.importScripts) ? "auto" : ""
	);

	// IGNORE(output.hashFunction): Rspack uses faster xxhash64 by default
	D(output, "hashFunction", "xxhash64");
	D(output, "hashDigest", "hex");
	// IGNORE(output.hashDigestLength): xxhash64 uses 16-bit hash
	D(output, "hashDigestLength", 16);
	D(output, "strictModuleErrorHandling", false);
	if (output.library) {
		F(output.library, "type", () => (output.module ? "module" : "var"));
	}
	F(output, "chunkFormat", () => {
		if (tp) {
			const helpMessage = isAffectedByBrowserslist
				? "Make sure that your 'browserslist' includes only platforms that support these features or select an appropriate 'target' to allow selecting a chunk format by default. Alternatively specify the 'output.chunkFormat' directly."
				: "Select an appropriate 'target' to allow selecting one by default, or specify the 'output.chunkFormat' directly.";
			if (output.module) {
				if (tp.dynamicImport) return "module";
				if (tp.document) return "array-push";
				throw new Error(
					`For the selected environment is no default ESM chunk format available:\nESM exports can be chosen when 'import()' is available.\nJSONP Array push can be chosen when 'document' is available.\n${helpMessage}`
				);
			}
			if (tp.document) return "array-push";
			if (tp.require) return "commonjs";
			if (tp.nodeBuiltins) return "commonjs";
			if (tp.importScripts) return "array-push";
			throw new Error(
				`For the selected environment is no default script chunk format available:\nJSONP Array push can be chosen when 'document' or 'importScripts' is available.\nCommonJs exports can be chosen when 'require' or node builtins are available.\n${helpMessage}`
			);
		}
		throw new Error(
			"Chunk format can't be selected by default when no target is specified"
		);
	});
	D(output, "asyncChunks", true);
	F(output, "chunkLoading", () => {
		if (tp) {
			switch (output.chunkFormat) {
				case "array-push":
					if (tp.document) return "jsonp";
					if (tp.importScripts) return "import-scripts";
					break;
				case "commonjs":
					if (tp.require) return "require";
					if (tp.nodeBuiltins) return "async-node";
					break;
				case "module":
					if (tp.dynamicImport) return "import";
					break;
			}
			if (
				tp.require === null ||
				tp.nodeBuiltins === null ||
				tp.document === null ||
				tp.importScripts === null
			) {
				return "universal";
			}
		}
		return false;
	});
	F(output, "workerChunkLoading", () => {
		if (tp) {
			switch (output.chunkFormat) {
				case "array-push":
					if (tp.importScriptsInWorker) return "import-scripts";
					break;
				case "commonjs":
					if (tp.require) return "require";
					if (tp.nodeBuiltins) return "async-node";
					break;
				case "module":
					if (tp.dynamicImportInWorker) return "import";
					break;
			}
			if (
				tp.require === null ||
				tp.nodeBuiltins === null ||
				tp.importScriptsInWorker === null
			) {
				return "universal";
			}
		}
		return false;
	});
	F(output, "wasmLoading", () => {
		if (tp) {
			if (tp.fetchWasm) return "fetch";
			if (tp.nodeBuiltins) return "async-node";
			if (tp.nodeBuiltins === null || tp.fetchWasm === null) {
				return "universal";
			}
		}
		return false;
	});
	F(output, "workerWasmLoading", () => output.wasmLoading);
	F(output, "globalObject", () => {
		if (tp) {
			if (tp.global) return "global";
			if (tp.globalThis) return "globalThis";
		}
		return "self";
	});
	D(output, "importFunctionName", "import");
	D(output, "importMetaName", "import.meta");
	// IGNORE(output.clean): The default value of `output.clean` in webpack is undefined, but it has the same effect as false.
	F(output, "clean", () => !!output.clean);
	D(output, "crossOriginLoading", false);
	D(output, "workerPublicPath", "");
	D(output, "sourceMapFilename", "[file].map[query]");
	F(output, "scriptType", () => (output.module ? "module" : false));
	D(output, "charset", !futureDefaults);
	D(output, "chunkLoadTimeout", 120000);

	const { trustedTypes } = output;
	if (trustedTypes) {
		F(
			trustedTypes,
			"policyName",
			() =>
				output.uniqueName!.replace(/[^a-zA-Z0-9\-#=_/@.%]+/g, "_") || "webpack"
		);
		D(trustedTypes, "onPolicyCreationFailure", "stop");
	}

	const forEachEntry = (fn: (desc: EntryDescriptionNormalized) => void) => {
		if (typeof entry === "function") {
			return;
		}
		for (const name of Object.keys(entry)) {
			fn(entry[name]);
		}
	};
	A(output, "enabledLibraryTypes", () => {
		const enabledLibraryTypes = [];
		if (output.library) {
			enabledLibraryTypes.push(output.library.type);
		}
		forEachEntry(desc => {
			if (desc.library) {
				enabledLibraryTypes.push(desc.library.type);
			}
		});
		return enabledLibraryTypes;
	});
	A(output, "enabledChunkLoadingTypes", () => {
		const enabledChunkLoadingTypes = new Set<string>();
		if (output.chunkLoading) {
			enabledChunkLoadingTypes.add(output.chunkLoading);
		}
		if (output.workerChunkLoading) {
			enabledChunkLoadingTypes.add(output.workerChunkLoading);
		}
		forEachEntry(desc => {
			if (desc.chunkLoading) {
				enabledChunkLoadingTypes.add(desc.chunkLoading);
			}
		});
		return Array.from(enabledChunkLoadingTypes);
	});
	A(output, "enabledWasmLoadingTypes", () => {
		const enabledWasmLoadingTypes = new Set<string>();
		if (output.wasmLoading) {
			enabledWasmLoadingTypes.add(output.wasmLoading);
		}
		if (output.workerWasmLoading) {
			enabledWasmLoadingTypes.add(output.workerWasmLoading);
		}
		// forEachEntry(desc => {
		// 	if (desc.wasmLoading) {
		// 		enabledWasmLoadingTypes.add(desc.wasmLoading);
		// 	}
		// });
		return Array.from(enabledWasmLoadingTypes);
	});

	const environment = output.environment!;
	const optimistic = (v?: boolean) => v || v === undefined;
	const conditionallyOptimistic = (v?: boolean, c?: boolean) =>
		(v === undefined && c) || v;

	F(environment, "globalThis", () => tp?.globalThis);
	F(environment, "bigIntLiteral", () => tp && optimistic(tp.bigIntLiteral));
	F(environment, "const", () => tp && optimistic(tp.const));
	F(environment, "arrowFunction", () => tp && optimistic(tp.arrowFunction));
	F(environment, "asyncFunction", () => tp && optimistic(tp.asyncFunction));
	F(environment, "forOf", () => tp && optimistic(tp.forOf));
	F(environment, "destructuring", () => tp && optimistic(tp.destructuring));
	F(
		environment,
		"optionalChaining",
		() => tp && optimistic(tp.optionalChaining)
	);
	F(
		environment,
		"nodePrefixForCoreModules",
		() => tp && optimistic(tp.nodePrefixForCoreModules)
	);
	F(environment, "templateLiteral", () => tp && optimistic(tp.templateLiteral));
	F(environment, "dynamicImport", () =>
		conditionallyOptimistic(tp?.dynamicImport, output.module)
	);
	F(environment, "dynamicImportInWorker", () =>
		conditionallyOptimistic(tp?.dynamicImportInWorker, output.module)
	);
	F(environment, "module", () =>
		conditionallyOptimistic(tp?.module, output.module)
	);
	F(environment, "document", () => tp && optimistic(tp.document));
};

const applyExternalsPresetsDefaults = (
	externalsPresets: ExternalsPresets,
	{ targetProperties }: { targetProperties: any }
) => {
	D(externalsPresets, "web", targetProperties?.web);
	D(externalsPresets, "node", targetProperties?.node);
	D(externalsPresets, "electron", targetProperties?.electron);
	D(
		externalsPresets,
		"electronMain",
		targetProperties?.electron && targetProperties.electronMain
	);
	D(
		externalsPresets,
		"electronPreload",
		targetProperties?.electron && targetProperties.electronPreload
	);
	D(
		externalsPresets,
		"electronRenderer",
		targetProperties?.electron && targetProperties.electronRenderer
	);
	D(externalsPresets, "nwjs", targetProperties?.nwjs);
};

const applyLoaderDefaults = (
	loader: Loader,
	{ targetProperties, environment }: { targetProperties: any; environment: any }
) => {
	F(loader, "target", () => {
		if (targetProperties) {
			if (targetProperties.electron) {
				if (targetProperties.electronMain) return "electron-main";
				if (targetProperties.electronPreload) return "electron-preload";
				if (targetProperties.electronRenderer) return "electron-renderer";
				return "electron";
			}
			if (targetProperties.nwjs) return "nwjs";
			if (targetProperties.node) return "node";
			if (targetProperties.web) return "web";
		}
	});
	D(loader, "environment", environment);
};

const applyNodeDefaults = (
	node: Node,
	{
		outputModule,
		targetProperties
	}: { targetProperties: any; outputModule?: boolean }
) => {
	if (node === false) return;

	// IGNORE(node.global): The default value of `global` is determined by `futureDefaults` in webpack.
	F(node, "global", () => {
		if (targetProperties?.global) return false;
		return "warn";
	});
	// IGNORE(node.__dirname): The default value of `__dirname` is determined by `futureDefaults` in webpack.
	F(node, "__dirname", () => {
		if (targetProperties?.node)
			return outputModule ? "node-module" : "eval-only";
		return "warn-mock";
	});
	// IGNORE(node.__filename): The default value of `__filename` is determined by `futureDefaults` in webpack.
	F(node, "__filename", () => {
		if (targetProperties?.node)
			return outputModule ? "node-module" : "eval-only";
		return "warn-mock";
	});
};

const applyPerformanceDefaults = (
	performance: Performance,
	{ production }: { production: boolean }
) => {
	if (performance === false) return;
	D(performance, "maxAssetSize", 250000);
	D(performance, "maxEntrypointSize", 250000);
	F(performance, "hints", () => (production ? "warning" : false));
};

const applyOptimizationDefaults = (
	optimization: Optimization,
	{
		production,
		development,
		css
	}: { production: boolean; development: boolean; css: boolean }
) => {
	D(optimization, "removeAvailableModules", false);
	D(optimization, "removeEmptyChunks", true);
	D(optimization, "mergeDuplicateChunks", true);
	F(optimization, "moduleIds", (): "natural" | "named" | "deterministic" => {
		if (production) return "deterministic";
		if (development) return "named";
		return "natural";
	});
	F(optimization, "chunkIds", (): "natural" | "named" | "deterministic" => {
		if (production) return "deterministic";
		if (development) return "named";
		return "natural";
	});
	F(optimization, "sideEffects", () => (production ? true : "flag"));
	D(optimization, "mangleExports", production);
	D(optimization, "providedExports", true);
	D(optimization, "usedExports", production);
	D(optimization, "innerGraph", production);
	D(optimization, "emitOnErrors", !production);
	D(optimization, "runtimeChunk", false);
	D(optimization, "realContentHash", production);
	// IGNORE(optimization.avoidEntryIife): to update the default value of webpack and bump webpack version in Rspack.
	D(optimization, "avoidEntryIife", false);
	D(optimization, "minimize", production);
	D(optimization, "concatenateModules", production);
	// IGNORE(optimization.minimizer): Rspack use `SwcJsMinimizerRspackPlugin` and `LightningCssMinimizerRspackPlugin` by default
	A(optimization, "minimizer", () => [
		new SwcJsMinimizerRspackPlugin(),
		new LightningCssMinimizerRspackPlugin()
	]);
	F(optimization, "nodeEnv", () => {
		if (production) return "production";
		if (development) return "development";
		return false;
	});
	const { splitChunks } = optimization;
	if (splitChunks) {
		// IGNORE(optimization.splitChunks.defaultSizeTypes): Rspack enables `experiments.css` by default currently
		A(splitChunks, "defaultSizeTypes", () =>
			css ? ["javascript", "css", "unknown"] : ["javascript", "unknown"]
		);
		D(splitChunks, "hidePathInfo", production);
		D(splitChunks, "chunks", "async");
		D(splitChunks, "usedExports", optimization.usedExports === true);
		D(splitChunks, "minChunks", 1);
		F(splitChunks, "minSize", () => (production ? 20000 : 10000));
		// F(splitChunks, "minRemainingSize", () => (development ? 0 : undefined));
		// F(splitChunks, "enforceSizeThreshold", () => (production ? 50000 : 30000));
		F(splitChunks, "maxAsyncRequests", () =>
			production ? 30 : Number.POSITIVE_INFINITY
		);
		F(splitChunks, "maxInitialRequests", () =>
			production ? 30 : Number.POSITIVE_INFINITY
		);
		D(splitChunks, "automaticNameDelimiter", "-");
		const { cacheGroups } = splitChunks;
		if (cacheGroups) {
			F(cacheGroups, "default", () => ({
				idHint: "",
				reuseExistingChunk: true,
				minChunks: 2,
				priority: -20
			}));
			F(cacheGroups, "defaultVendors", () => ({
				idHint: "vendors",
				reuseExistingChunk: true,
				test: /[\\/]node_modules[\\/]/i,
				priority: -10
			}));
		}
	}
};

const getResolveLoaderDefaults = () => {
	const resolveOptions: ResolveOptions = {
		conditionNames: ["loader", "require", "node"],
		exportsFields: ["exports"],
		mainFields: ["loader", "main"],
		extensions: [".js"],
		mainFiles: ["index"]
	};

	return resolveOptions;
};

// The values are aligned with webpack
// https://github.com/webpack/webpack/blob/b9fb99c63ca433b24233e0bbc9ce336b47872c08/lib/config/defaults.js#L1431
const getResolveDefaults = ({
	context,
	targetProperties,
	mode,
	css
}: {
	context: string;
	targetProperties: any;
	mode?: Mode;
	css: boolean;
}) => {
	const conditions = ["webpack"];

	conditions.push(mode === "development" ? "development" : "production");

	if (targetProperties) {
		if (targetProperties.webworker) conditions.push("worker");
		if (targetProperties.node) conditions.push("node");
		if (targetProperties.web) conditions.push("browser");
		if (targetProperties.electron) conditions.push("electron");
		if (targetProperties.nwjs) conditions.push("nwjs");
	}
	const jsExtensions = [".js", ".json", ".wasm"];

	const tp = targetProperties;

	const browserField =
		tp?.web && (!tp.node || (tp.electron && tp.electronRenderer));
	const aliasFields = browserField ? ["browser"] : [];
	const mainFields = browserField
		? ["browser", "module", "..."]
		: ["module", "..."];

	const cjsDeps = () => ({
		aliasFields,
		mainFields,
		conditionNames: ["require", "module", "..."],
		extensions: [...jsExtensions]
	});

	const esmDeps = () => ({
		aliasFields,
		mainFields,
		conditionNames: ["import", "module", "..."],
		extensions: [...jsExtensions]
	});

	const resolveOptions: ResolveOptions = {
		// enable pnp only in pnp environment, see https://yarnpkg.com/advanced/pnpapi#processversionspnp
		// IGNORE(resolve.pnp): Rspack use `resolve.enable` to enable Yarn PnP feature
		pnp: getPnpDefault(),
		modules: ["node_modules"],
		conditionNames: conditions,
		mainFiles: ["index"],
		extensions: [],
		aliasFields: [],
		exportsFields: ["exports"],
		roots: [context],
		mainFields: ["main"],
		importsFields: ["imports"],
		byDependency: {
			wasm: esmDeps(),
			esm: esmDeps(),
			url: {
				preferRelative: true
			},
			worker: {
				...esmDeps(),
				preferRelative: true
			},
			commonjs: cjsDeps(),
			amd: cjsDeps(),
			// for backward-compat: loadModule
			// loader: cjsDeps(),
			// for backward-compat: Custom Dependency and getResolve without dependencyType
			unknown: cjsDeps()
		}
	};

	if (css) {
		const styleConditions = [];

		styleConditions.push("webpack");
		styleConditions.push(mode === "development" ? "development" : "production");
		styleConditions.push("style");

		// IGNORE(resolve.byDependency.css-import): Rspack enables `experiments.css` by default currently
		resolveOptions.byDependency!["css-import"] = {
			// We avoid using any main files because we have to be consistent with CSS `@import`
			// and CSS `@import` does not handle `main` files in directories,
			// you should always specify the full URL for styles
			mainFiles: [],
			mainFields: ["style", "..."],
			conditionNames: styleConditions,
			extensions: [".css"],
			preferRelative: true
		};
	}

	return resolveOptions;
};

const D = <T, P extends keyof T>(obj: T, prop: P, value: T[P]) => {
	if (obj[prop] === undefined) {
		obj[prop] = value;
	}
};

const F = <T, P extends keyof T>(obj: T, prop: P, factory: () => T[P]) => {
	if (obj[prop] === undefined) {
		obj[prop] = factory();
	}
};

const A = <T, P extends keyof T>(
	obj: T,
	prop: P,
	factory: () => T[P]
): void => {
	const value = obj[prop];
	if (value === undefined) {
		obj[prop] = factory();
	} else if (Array.isArray(value)) {
		let newArray = undefined;
		for (let i = 0; i < value.length; i++) {
			const item = value[i];
			if (item === "...") {
				if (newArray === undefined) {
					newArray = value.slice(0, i);
					obj[prop] = newArray as any;
				}
				const items = factory();
				if (items !== undefined) {
					for (const item of items as any) {
						newArray.push(item);
					}
				}
			} else if (newArray !== undefined) {
				newArray.push(item);
			}
		}
	}
};

export const getPnpDefault = () => {
	return !!process.versions.pnp;
};
