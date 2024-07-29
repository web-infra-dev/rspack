/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/config/target.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import { memoize } from "../util/memoize";
import * as browserslistTargetHandler from "./browserslistTargetHandler";

const getBrowserslistTargetHandler = memoize(() => browserslistTargetHandler);

/**
 * @param context the context directory
 * @returns default target
 */
export const getDefaultTarget = (context: string): "browserslist" | "web" => {
	const browsers = getBrowserslistTargetHandler().load(null, context);
	return browsers ? "browserslist" : "web";
};

export type PlatformTargetProperties = {
	/** web platform, importing of http(s) and std: is available */
	web: boolean | null;
	/** browser platform, running in a normal web browser */
	browser: boolean | null;
	/** (Web)Worker platform, running in a web/shared/service worker */
	webworker: boolean | null;
	/** node platform, require of node built-in modules is available */
	node: boolean | null;
	/** nwjs platform, require of legacy nw.gui is available */
	nwjs: boolean | null;
	/** electron platform, require of some electron built-in modules is available */
	electron: boolean | null;
};

export type ElectronContextTargetProperties = {
	/**  in main context */
	electronMain: boolean | null;
	/**  in preload context */
	electronPreload: boolean | null;
	/**  in renderer context with node integration */
	electronRenderer: boolean | null;
};

export type ApiTargetProperties = {
	/**  has require function available */
	require: boolean | null;
	/**  has node.js built-in modules available */
	nodeBuiltins: boolean | null;
	/**  node.js allows to use `node:` prefix for core modules */
	nodePrefixForCoreModules: boolean | null;
	/**  has document available (allows script tags) */
	document: boolean | null;
	/**  has importScripts available */
	importScripts: boolean | null;
	/**  has importScripts available when creating a worker */
	importScriptsInWorker: boolean | null;
	/**  has fetch function available for WebAssembly */
	fetchWasm: boolean | null;
	/**  has global variable available */
	global: boolean | null;
};

export type EcmaTargetProperties = {
	/**  has globalThis variable available */
	globalThis: boolean | null;
	/**  big int literal syntax is available */
	bigIntLiteral: boolean | null;
	/**  const and let variable declarations are available */
	const: boolean | null;
	/**  arrow functions are available */
	arrowFunction: boolean | null;
	/**  for of iteration is available */
	forOf: boolean | null;
	/**  destructuring is available */
	destructuring: boolean | null;
	/**  async import() is available */
	dynamicImport: boolean | null;
	/**  async import() is available when creating a worker */
	dynamicImportInWorker: boolean | null;
	/**  ESM syntax is available (when in module) */
	module: boolean | null;
	/**  optional chaining is available */
	optionalChaining: boolean | null;
	/**  template literal is available */
	templateLiteral: boolean | null;
	/**  async functions and await are available */
	asyncFunction: boolean | null;
};

// type TargetProperties =
// 	| PlatformTargetProperties
// 	| ApiTargetProperties
// 	| EcmaTargetProperties
// 	| (PlatformTargetProperties & ApiTargetProperties)
// 	| (PlatformTargetProperties & EcmaTargetProperties)
// 	| (ApiTargetProperties & EcmaTargetProperties);

type Never<T> = { [P in keyof T]?: never };
type Mix<A, B> = (A & Never<B>) | (Never<A> & B) | (A & B);

type TargetProperties = Mix<
	Mix<PlatformTargetProperties, ElectronContextTargetProperties>,
	Mix<ApiTargetProperties, EcmaTargetProperties>
>;

/**
 * @param major major version
 * @param minor minor version
 * @returns check if version is greater or equal
 */
const versionDependent = (
	major: string,
	minor: string | undefined
): ((vMajor: number, vMinor?: number) => boolean | undefined) => {
	if (!major) {
		return () => undefined;
	}
	const nMajor = +major;
	const nMinor = minor ? +minor : 0;

	return (vMajor, vMinor = 0) => {
		return nMajor > vMajor || (nMajor === vMajor && nMinor >= vMinor);
	};
};

const TARGETS: Array<
	[string, string, RegExp, (...args: string[]) => Partial<TargetProperties>]
> = [
	[
		"browserslist / browserslist:env / browserslist:query / browserslist:path-to-config / browserslist:path-to-config:env",
		"Resolve features from browserslist. Will resolve browserslist config automatically. Only browser or node queries are supported (electron is not supported). Examples: 'browserslist:modern' to use 'modern' environment from browserslist config",
		/^browserslist(?::(.+))?$/,
		(rest, context) => {
			const browserslistTargetHandler = getBrowserslistTargetHandler();
			const browsers = browserslistTargetHandler.load(
				rest ? rest.trim() : null,
				context
			);
			if (!browsers) {
				throw new Error(`No browserslist config found to handle the 'browserslist' target.
See https://github.com/browserslist/browserslist#queries for possible ways to provide a config.
The recommended way is to add a 'browserslist' key to your package.json and list supported browsers (resp. node.js versions).
You can also more options via the 'target' option: 'browserslist' / 'browserslist:env' / 'browserslist:query' / 'browserslist:path-to-config' / 'browserslist:path-to-config:env'`);
			}

			return browserslistTargetHandler.resolve(browsers);
		}
	],
	[
		"web",
		"Web browser.",
		/^web$/,
		() => {
			return {
				web: true,
				browser: true,
				webworker: null,
				node: false,
				electron: false,
				nwjs: false,

				document: true,
				importScriptsInWorker: true,
				fetchWasm: true,
				nodeBuiltins: false,
				importScripts: false,
				require: false,
				global: false
			};
		}
	],
	[
		"webworker",
		"Web Worker, SharedWorker or Service Worker.",
		/^webworker$/,
		() => {
			return {
				web: true,
				browser: true,
				webworker: true,
				node: false,
				electron: false,
				nwjs: false,

				importScripts: true,
				importScriptsInWorker: true,
				fetchWasm: true,
				nodeBuiltins: false,
				require: false,
				document: false,
				global: false
			};
		}
	],
	[
		"[async-]node[X[.Y]]",
		"Node.js in version X.Y. The 'async-' prefix will load chunks asynchronously via 'fs' and 'vm' instead of 'require()'. Examples: node14.5, async-node10.",
		/^(async-)?node((\d+)(?:\.(\d+))?)?$/,
		(asyncFlag, _, major, minor) => {
			const v = versionDependent(major, minor);
			// see https://node.green/
			return {
				node: true,
				electron: false,
				nwjs: false,
				web: false,
				webworker: false,
				browser: false,

				require: !asyncFlag,
				nodeBuiltins: true,
				// v16.0.0, v14.18.0
				nodePrefixForCoreModules: +major < 15 ? v(14, 18) : v(16),
				global: true,
				document: false,
				fetchWasm: false,
				importScripts: false,
				importScriptsInWorker: false,

				globalThis: v(12),
				const: v(6),
				templateLiteral: v(4),
				optionalChaining: v(14),
				arrowFunction: v(6),
				asyncFunction: v(7, 6),
				forOf: v(5),
				destructuring: v(6),
				bigIntLiteral: v(10, 4),
				dynamicImport: v(12, 17),
				dynamicImportInWorker: major ? false : undefined,
				module: v(12, 17)
			};
		}
	],
	[
		"electron[X[.Y]]-main/preload/renderer",
		"Electron in version X.Y. Script is running in main, preload resp. renderer context.",
		/^electron((\d+)(?:\.(\d+))?)?-(main|preload|renderer)$/,
		(_, major, minor, context) => {
			const v = versionDependent(major, minor);
			// see https://node.green/ + https://github.com/electron/releases
			return {
				node: true,
				electron: true,
				web: context !== "main",
				webworker: false,
				browser: false,
				nwjs: false,

				electronMain: context === "main",
				electronPreload: context === "preload",
				electronRenderer: context === "renderer",

				global: true,
				nodeBuiltins: true,
				// 15.0.0	- Node.js	v16.5
				// 14.0.0 - Mode.js v14.17, but prefixes only since v14.18
				nodePrefixForCoreModules: v(15),

				require: true,
				document: context === "renderer",
				fetchWasm: context === "renderer",
				importScripts: false,
				importScriptsInWorker: true,

				globalThis: v(5),
				const: v(1, 1),
				templateLiteral: v(1, 1),
				optionalChaining: v(8),
				arrowFunction: v(1, 1),
				asyncFunction: v(1, 7),
				forOf: v(0, 36),
				destructuring: v(1, 1),
				bigIntLiteral: v(4),
				dynamicImport: v(11),
				dynamicImportInWorker: major ? false : undefined,
				module: v(11)
			};
		}
	],
	[
		"nwjs[X[.Y]] / node-webkit[X[.Y]]",
		"NW.js in version X.Y.",
		/^(?:nwjs|node-webkit)((\d+)(?:\.(\d+))?)?$/,
		(_, major, minor) => {
			const v = versionDependent(major, minor);
			// see https://node.green/ + https://github.com/nwjs/nw.js/blob/nw48/CHANGELOG.md
			return {
				node: true,
				web: true,
				nwjs: true,
				webworker: null,
				browser: false,
				electron: false,

				global: true,
				nodeBuiltins: true,
				document: false,
				importScriptsInWorker: false,
				fetchWasm: false,
				importScripts: false,
				require: false,

				globalThis: v(0, 43),
				const: v(0, 15),
				templateLiteral: v(0, 13),
				optionalChaining: v(0, 44),
				arrowFunction: v(0, 15),
				asyncFunction: v(0, 21),
				forOf: v(0, 13),
				destructuring: v(0, 15),
				bigIntLiteral: v(0, 32),
				dynamicImport: v(0, 43),
				dynamicImportInWorker: major ? false : undefined,
				module: v(0, 43)
			};
		}
	],
	[
		"esX",
		"EcmaScript in this version. Examples: es2020, es5.",
		/^es(\d+)$/,
		version => {
			let v = +version;
			if (v < 1000) v = v + 2009;
			return {
				const: v >= 2015,
				templateLiteral: v >= 2015,
				optionalChaining: v >= 2020,
				arrowFunction: v >= 2015,
				forOf: v >= 2015,
				destructuring: v >= 2015,
				module: v >= 2015,
				asyncFunction: v >= 2017,
				globalThis: v >= 2020,
				bigIntLiteral: v >= 2020,
				dynamicImport: v >= 2020,
				dynamicImportInWorker: v >= 2020
			};
		}
	]
];

/**
 * @param target the target
 * @param context the context directory
 * @returns target properties
 */
export const getTargetProperties = (
	target: string,
	context: string
): TargetProperties => {
	for (const [, , regExp, handler] of TARGETS) {
		const match = regExp.exec(target);
		if (match) {
			const [, ...args] = match;
			const result = handler(...args, context);
			if (result) return result as TargetProperties;
		}
	}
	throw new Error(
		`Unknown target '${target}'. The following targets are supported:\n${TARGETS.map(
			([name, description]) => `* ${name}: ${description}`
		).join("\n")}`
	);
};

const mergeTargetProperties = (
	targetProperties: TargetProperties[]
): TargetProperties => {
	const keys = new Set<keyof TargetProperties>();
	for (const tp of targetProperties) {
		for (const key of Object.keys(tp) as Array<keyof TargetProperties>) {
			keys.add(key);
		}
	}

	const result: Partial<TargetProperties> = {};
	for (const key of keys) {
		let hasTrue = false;
		let hasFalse = false;
		for (const tp of targetProperties) {
			const value = tp[key];
			switch (value) {
				case true:
					hasTrue = true;
					break;
				case false:
					hasFalse = true;
					break;
			}
		}
		if (hasTrue || hasFalse) result[key] = hasFalse && hasTrue ? null : hasTrue;
	}
	return result as TargetProperties;
};

/**
 * @param targets the targets
 * @param context the context directory
 * @returns target properties
 */
export const getTargetsProperties = (targets: string[], context: string) => {
	return mergeTargetProperties(
		targets.map(t => getTargetProperties(t, context))
	);
};
