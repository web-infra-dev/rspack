import path from "path";
import fs from "fs";
import {
	getDefaultTarget,
	getTargetProperties,
	getTargetsProperties
} from "./target";
import type {
	Context,
	Experiments,
	InfrastructureLogging,
	Mode,
	ModuleOptions,
	Node,
	Optimization,
	OutputNormalized,
	ResolveOptions,
	RspackOptionsNormalized,
	RuleSetRules,
	SnapshotOptions
} from "./types";
import { cleverMerge } from "../util/cleverMerge";
import assert from "assert";
import { isNil } from "../util";

export const applyRspackOptionsDefaults = (
	options: RspackOptionsNormalized
) => {
	F(options, "context", () => process.cwd());
	F(options, "target", () => {
		return getDefaultTarget(options.context!);
	});

	const { mode, name, target } = options;
	assert(!isNil(target));

	let targetProperties =
		target === false
			? false
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

	F(options, "devtool", () => false as const);
	D(options, "watch", false);

	applyExperimentsDefaults(options.experiments);

	F(options, "cache", () => development);

	applySnapshotDefaults(options.snapshot, { production });

	applyModuleDefaults(options.module);

	applyOutputDefaults(options.output, {
		context: options.context!,
		targetProperties
	});

	// TODO: align with webpack
	D(options, "externalsType", undefined);

	applyNodeDefaults(options.node, { targetProperties });

	applyOptimizationDefaults(options.optimization, { production, development });

	options.resolve = cleverMerge(
		getResolveDefaults({
			targetProperties,
			mode: options.mode
		}),
		options.resolve
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

const applyExperimentsDefaults = (experiments: Experiments) => {
	D(experiments, "incrementalRebuild", false);
	D(experiments, "lazyCompilation", false);
};

const applySnapshotDefaults = (
	snapshot: SnapshotOptions,
	{ production }: { production: boolean }
) => {
	F(snapshot, "module", () =>
		production
			? { timestamp: true, hash: true }
			: { timestamp: true, hash: false }
	);
	F(snapshot, "resolve", () =>
		production
			? { timestamp: true, hash: true }
			: { timestamp: true, hash: false }
	);
};

const applyModuleDefaults = (module: ModuleOptions) => {
	F(module.parser!, "asset", () => ({}));
	F(module.parser!.asset!, "dataUrlCondition", () => ({}));
	if (typeof module.parser!.asset!.dataUrlCondition === "object") {
		D(module.parser!.asset!.dataUrlCondition, "maxSize", 8096);
	}

	A(module, "defaultRules", () => {
		const esm = {
			type: "javascript/esm"
		};
		const commonjs = {
		  // TODO: this is "javascript/dynamic" in webpack
			type: "javascript/auto"
		};
		const rules: RuleSetRules = [
			{
				test: /\.json$/i,
				type: "json"
			},
			{
				test: /\.mjs$/i,
				...esm
			},
			{
				test: /\.js$/i,
				...esm
			},
			{
				test: /\.cjs$/i,
				...commonjs
			},
			{
				test: /\.js$/i,
				...commonjs
			}
		];
		const cssRule = {
			type: "css",
			resolve: {
				preferRelative: true
			}
		};
		const cssModulesRule = {
			type: "css/module"
		};
		rules.push({
			test: /\.css$/i,
			oneOf: [
				{
					test: /\.module\.css$/i,
					...cssModulesRule
				},
				{
					...cssRule
				}
			]
		});
		return rules;
	});
};

const applyOutputDefaults = (
	output: OutputNormalized,
	{ context, targetProperties: tp }: { context: Context; targetProperties: any }
) => {
	F(output, "uniqueName", () => {
		const pkgPath = path.resolve(context, "package.json");
		try {
			const packageInfo = JSON.parse(fs.readFileSync(pkgPath, "utf-8"));
			return packageInfo.name || "";
		} catch (e: any) {
			if (e.code !== "ENOENT") {
				e.message += `\nwhile determining default 'output.uniqueName' from 'name' in ${pkgPath}`;
				throw e;
			}
			return "";
		}
	});

	D(output, "filename", "[name].js");
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
	D(output, "assetModuleFilename", "[hash][ext][query]");
	F(output, "path", () => path.join(process.cwd(), "dist"));
	D(
		output,
		"publicPath",
		tp && (tp.document || tp.importScripts) ? "auto" : ""
	);
	D(output, "strictModuleErrorHandling", false);
};

const applyNodeDefaults = (
	node: Node,
	{ targetProperties }: { targetProperties: any }
) => {
	F(node, "__dirname", () => {
		if (targetProperties && targetProperties.node) return "eval-only";
		return "warn-mock";
	});
};

const applyOptimizationDefaults = (
	optimization: Optimization,
	{ production, development }: { production: boolean; development: boolean }
) => {
	D(optimization, "removeAvailableModules", false);
	F(optimization, "moduleIds", () => {
		if (production) return "deterministic";
		return "named";
	});
	F(optimization, "sideEffects", () => (production ? true : "flag"));
	D(optimization, "runtimeChunk", false);
	D(optimization, "minimize", production);
	A(optimization, "minimizer", () => []);
	const { splitChunks } = optimization;
	if (splitChunks) {
		D(splitChunks, "chunks", "async");
		D(splitChunks, "minChunks", 1);
		F(splitChunks, "minSize", () => (production ? 20000 : 10000));
		F(splitChunks, "minRemainingSize", () => (development ? 0 : undefined));
		F(splitChunks, "enforceSizeThreshold", () => (production ? 50000 : 30000));
		F(splitChunks, "maxAsyncRequests", () => (production ? 30 : Infinity));
		F(splitChunks, "maxInitialRequests", () => (production ? 30 : Infinity));
		const cacheGroups = splitChunks.cacheGroups!;
		// TODO: default and defaultVendors cacheGroups
	}
};

const getResolveDefaults = ({
	targetProperties,
	mode
}: {
	targetProperties: any;
	mode?: Mode;
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

	const jsExtensions = [".tsx", ".jsx", ".ts", ".js", ".json", ".d.ts"];

	const tp = targetProperties;
	const browserField =
		tp && tp.web && (!tp.node || (tp.electron && tp.electronRenderer));

	const resolveOptions: ResolveOptions = {
		modules: ["node_modules"],
		// TODO: align with webpack, we need resolve.byDependency!
		// conditionNames: undefined,
		mainFiles: ["index"],
		// TODO: align with webpack
		extensions: [...jsExtensions],
		browserField,
		// TODO: align with webpack, we need resolve.byDependency!
		mainFields: [browserField && "browser", "module", "main"].filter(Boolean)
	};

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
					// @ts-expect-error
					obj[prop] = newArray;
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
