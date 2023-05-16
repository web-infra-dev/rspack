import * as path from "path";

import type {
	RawBuiltins,
	RawBannerConfig,
	RawHtmlPluginConfig,
	RawDecoratorOptions,
	RawMinification,
	RawReactOptions,
	RawProgressPluginConfig,
	RawPostCssConfig,
	RawCopyConfig,
	RawPattern,
	RawPresetEnv,
	RawPluginImportConfig,
	RawCssModulesConfig,
	RawRelayConfig,
	RawCodeGeneration
} from "@rspack/binding";
import { loadConfig } from "browserslist";
import { getBannerConditions } from "./adapter";
import { Optimization } from "..";

export type BuiltinsHtmlPluginConfig = Omit<RawHtmlPluginConfig, "meta"> & {
	meta?: Record<string, string | Record<string, string>>;
};

export type EmotionConfigImportMap = {
	[packageName: string]: {
		[exportName: string]: {
			canonicalImport?: [string, string];
		};
	};
};

export type EmotionConfig =
	| boolean
	| {
			sourceMap?: boolean;
			autoLabel?: "never" | "dev-only" | "always";
			labelFormat?: string;
			importMap?: EmotionConfigImportMap;
	  };

export type CssPluginConfig = {
	modules?: Partial<RawCssModulesConfig>;
};

export type MinificationConfig = {
	passes?: number;
	dropConsole?: boolean;
	pureFuncs?: Array<string>;
	extractComments?: boolean | RegExp;
};

export interface Builtins {
	css?: CssPluginConfig;
	postcss?: RawPostCssConfig;
	treeShaking?: boolean | "module";
	progress?: boolean | RawProgressPluginConfig;
	react?: RawReactOptions;
	noEmitAssets?: boolean;
	define?: Record<string, string | boolean | undefined>;
	provide?: Record<string, string | string[]>;
	html?: Array<BuiltinsHtmlPluginConfig>;
	decorator?: boolean | Partial<RawDecoratorOptions>;
	minifyOptions?: MinificationConfig;
	emotion?: EmotionConfig;
	presetEnv?: Partial<RawBuiltins["presetEnv"]>;
	polyfill?: boolean;
	devFriendlySplitChunks?: boolean;
	copy?: CopyConfig;
	banner?: BannerConfigs;
	pluginImport?: PluginImportConfig[];
	relay?: RelayConfig;
	codeGeneration?: Partial<RawCodeGeneration>;
}

export type PluginImportConfig = {
	libraryName: string;
	libraryDirectory?: string;
	customName?: string;
	customStyleName?: string;
	style?: string | boolean;
	styleLibraryDirectory?: string;
	camelToDashComponentName?: boolean;
	transformToDefaultImport?: boolean;
	ignoreEsComponent?: Array<string>;
	ignoreStyleComponent?: Array<string>;
};

export type CopyConfig = {
	patterns:
		| string[]
		| ({
				from: string;
		  } & Partial<RawPattern>)[];
};

export type BannerCondition = string | RegExp;

export type BannerConditions = BannerCondition | BannerCondition[];

type BannerConfig =
	| string
	| {
			banner: string;
			entryOnly?: boolean;
			footer?: boolean;
			raw?: boolean;
			test?: BannerConditions;
			exclude?: BannerConditions;
			include?: BannerConditions;
	  };

export type BannerConfigs = BannerConfig | BannerConfig[];

export type RelayConfig = boolean | RawRelayConfig;

export type ResolvedBuiltins = Omit<RawBuiltins, "html"> & {
	html?: Array<BuiltinsHtmlPluginConfig>;
	emotion?: string;
};

function resolvePresetEnv(
	presetEnv: Builtins["presetEnv"],
	context: string
): RawPresetEnv | undefined {
	if (!presetEnv) {
		return undefined;
	}
	return {
		targets: presetEnv?.targets ?? loadConfig({ path: context }) ?? [],
		mode: presetEnv?.mode,
		coreJs: presetEnv?.coreJs
	};
}

function resolvePluginImport(
	pluginImport?: PluginImportConfig[]
): RawPluginImportConfig[] | undefined {
	if (!pluginImport) {
		return undefined;
	}

	return pluginImport.map(config => {
		const rawConfig: RawPluginImportConfig = {
			...config,
			style: {} // As babel-plugin-import style config is very flexible, we convert it to a more specific structure
		};

		if (typeof config.style === "boolean") {
			rawConfig.style!.bool = config.style;
		} else if (typeof config.style === "string") {
			const isTpl = config.style.includes("{{");
			rawConfig.style![isTpl ? "custom" : "css"] = config.style;
		}

		// This option will overrides the behavior of style
		if (config.styleLibraryDirectory) {
			rawConfig.style = { styleLibraryDirectory: config.styleLibraryDirectory };
		}

		return rawConfig;
	});
}

function resolveDefine(define: Builtins["define"]): RawBuiltins["define"] {
	// @ts-expect-error
	const entries = Object.entries(define).map(([key, value]) => {
		if (typeof value !== "string") {
			value = value === undefined ? "undefined" : JSON.stringify(value);
		}
		return [key, value];
	});
	return Object.fromEntries(entries);
}

function resolveTreeShaking(
	treeShaking: Builtins["treeShaking"],
	production: boolean
): RawBuiltins["treeShaking"] {
	return treeShaking !== undefined
		? treeShaking.toString()
		: production
		? "true"
		: "false";
}

function resolveProvide(
	provide: Builtins["provide"] = {}
): RawBuiltins["provide"] {
	const entries = Object.entries(provide).map(([key, value]) => {
		if (typeof value === "string") {
			value = [value];
		}
		return [key, value];
	});
	return Object.fromEntries(entries);
}

function resolveHtml(html: BuiltinsHtmlPluginConfig[]): RawHtmlPluginConfig[] {
	return html.map(c => {
		const meta: Record<string, Record<string, string>> = {};
		for (const key in c.meta) {
			const value = c.meta[key];
			if (typeof value === "string") {
				meta[key] = {
					name: key,
					content: value
				};
			}
		}
		return {
			...c,
			meta
		};
	});
}

function resolveDecorator(
	decorator: Builtins["decorator"]
): RawDecoratorOptions | undefined {
	if (decorator === false) {
		return undefined;
	}

	if (decorator === undefined || decorator === true) {
		decorator = {};
	}

	return Object.assign(
		{
			legacy: true,
			emitMetadata: true
		},
		decorator
	);
}

function resolveProgress(
	progress: Builtins["progress"]
): RawProgressPluginConfig | undefined {
	if (!progress) {
		return undefined;
	}

	if (progress === true) {
		progress = {};
	}

	return progress;
}

function resolveEmotion(
	emotion: Builtins["emotion"],
	isProduction: boolean
): string | undefined {
	if (!emotion) {
		return undefined;
	}

	if (emotion === true) {
		emotion = {};
	}

	const autoLabel = emotion?.autoLabel ?? "dev-only";

	const emotionConfig: Builtins["emotion"] = {
		enabled: true,
		// @ts-expect-error autoLabel is string for JavaScript interface, however is boolean for Rust interface
		autoLabel:
			autoLabel === "dev-only" ? !isProduction : autoLabel === "always",
		importMap: emotion?.importMap,
		labelFormat: emotion?.labelFormat ?? "[local]",
		sourcemap: isProduction ? false : emotion?.sourceMap ?? true
	};

	return JSON.stringify(emotionConfig);
}

function resolveCopy(copy?: Builtins["copy"]): RawCopyConfig | undefined {
	if (!copy) {
		return undefined;
	}

	const ret: RawCopyConfig = {
		patterns: []
	};

	ret.patterns = (copy.patterns || []).map(pattern => {
		if (typeof pattern === "string") {
			pattern = { from: pattern };
		}

		pattern.force ??= false;
		pattern.noErrorOnMissing ??= false;
		pattern.priority ??= 0;
		pattern.globOptions ??= {};

		return pattern as RawPattern;
	});

	return ret;
}

function resolveRelay(
	relay: RelayConfig,
	rootDir: string
): RawRelayConfig | undefined {
	if (!relay) {
		return undefined;
	}

	// Search relay config based on
	if (relay === true) {
		return (
			getRelayConfigFromProject(rootDir) || {
				language: "javascript"
			}
		);
	} else {
		return relay;
	}
}

function getRelayConfigFromProject(
	rootDir: string
): RawRelayConfig | undefined {
	for (const configName of [
		"relay.config.json",
		"relay.config.js",
		"package.json"
	]) {
		const configPath = path.join(rootDir, configName);
		try {
			let config = require(configPath) as
				| Partial<RawRelayConfig>
				| { relay?: Partial<RawRelayConfig> }
				| undefined;

			let finalConfig: Partial<RawRelayConfig> | undefined;
			if (configName === "package.json") {
				finalConfig = (config as { relay?: Partial<RawRelayConfig> })?.relay;
			} else {
				finalConfig = config as Partial<RawRelayConfig> | undefined;
			}

			if (finalConfig) {
				return {
					language: finalConfig.language!,
					artifactDirectory: finalConfig.artifactDirectory
				};
			}
		} catch (_) {}
	}
}

export function resolveBuiltinsOptions(
	builtins: Builtins,
	{
		contextPath,
		production,
		optimization
	}: { contextPath: string; production: boolean; optimization: Optimization }
): RawBuiltins {
	const presetEnv = resolvePresetEnv(builtins.presetEnv, contextPath);
	builtins.presetEnv ?? loadConfig({ path: contextPath }) ?? [];
	return {
		css: {
			modules: {
				localsConvention: "asIs",
				localIdentName: production ? "[hash]" : "[path][name][ext]__[local]",
				exportsOnly: false,
				...builtins.css?.modules
			}
		},
		postcss: { pxtorem: undefined, ...builtins.postcss },
		treeShaking: resolveTreeShaking(builtins.treeShaking, production),
		react: builtins.react ?? {},
		noEmitAssets: builtins.noEmitAssets ?? false,
		define: resolveDefine(builtins.define || {}),
		provide: resolveProvide(builtins.provide),
		html: resolveHtml(builtins.html || []),
		presetEnv,
		progress: resolveProgress(builtins.progress),
		decorator: resolveDecorator(builtins.decorator),
		minifyOptions: resolveMinifyOptions(builtins, optimization),
		emotion: resolveEmotion(builtins.emotion, production),
		devFriendlySplitChunks: builtins.devFriendlySplitChunks ?? false,
		copy: resolveCopy(builtins.copy),
		banner: resolveBanner(builtins.banner),
		pluginImport: resolvePluginImport(builtins.pluginImport),
		relay: builtins.relay
			? resolveRelay(builtins.relay, contextPath)
			: undefined,
		codeGeneration: resolveCodeGeneration(builtins)
	};
}

function resolveBannerConfig(bannerConfig: BannerConfig): RawBannerConfig {
	if (typeof bannerConfig === "string") {
		return {
			banner: bannerConfig
		};
	}

	return {
		...bannerConfig,
		test: getBannerConditions(bannerConfig.test),
		include: getBannerConditions(bannerConfig.include),
		exclude: getBannerConditions(bannerConfig.exclude)
	};
}

function resolveBanner(
	bannerConfigs?: BannerConfigs
): RawBannerConfig[] | undefined {
	if (!bannerConfigs) {
		return undefined;
	}

	if (Array.isArray(bannerConfigs)) {
		return bannerConfigs.map(resolveBannerConfig);
	}

	return [resolveBannerConfig(bannerConfigs)];
}

export function resolveMinifyOptions(
	builtins: Builtins,
	optimization: Optimization
): RawMinification | undefined {
	const disable_minify =
		!optimization.minimize ||
		optimization.minimizer?.some(item => item !== "...");

	if (disable_minify) {
		return undefined;
	}

	let extractComments = builtins.minifyOptions?.extractComments
		? String(builtins.minifyOptions.extractComments)
		: undefined;

	return {
		passes: 1,
		dropConsole: false,
		pureFuncs: [],
		...builtins.minifyOptions,
		extractComments
	};
}

export function resolveCodeGeneration(builtins: Builtins): RawCodeGeneration {
	if (!builtins.codeGeneration) {
		return { keepComments: Boolean(builtins.minifyOptions?.extractComments) };
	}
	return {
		keepComments: false,
		...builtins.codeGeneration
	};
}
