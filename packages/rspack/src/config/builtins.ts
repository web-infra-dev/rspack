import * as path from "path";

import type {
	RawBannerConfig,
	RawHtmlPluginConfig,
	RawDecoratorOptions,
	RawMinification,
	RawReactOptions,
	RawProgressPluginConfig,
	RawCopyConfig,
	RawPattern,
	RawPresetEnv,
	RawPluginImportConfig,
	RawRelayConfig,
	RawBannerConditions,
	RawBannerCondition,
	RawMinificationCondition,
	RawMinificationConditions
} from "@rspack/binding";
import { loadConfig } from "browserslist";
import { Optimization, RspackOptionsNormalized } from "..";
import {
	BannerPluginOptions,
	SwcJsMinimizerPluginOptions
} from "../builtin-plugin";
import { EmotionConfig } from "../builtin-plugin/depracate-by-swc-loader";

export type BuiltinsHtmlPluginConfig = Omit<RawHtmlPluginConfig, "meta"> & {
	meta?: Record<string, string | Record<string, string>>;
};

export interface Builtins {
	css?: CssPluginConfig;
	postcss?: any;
	treeShaking?: boolean | "module";
	progress?: boolean | RawProgressPluginConfig;
	react?: RawReactOptions;
	noEmitAssets?: boolean;
	define?: Record<string, string | boolean | undefined>;
	provide?: Record<string, string | string[]>;
	html?: Array<BuiltinsHtmlPluginConfig>;
	decorator?: boolean | Partial<RawDecoratorOptions>;
	minifyOptions?: SwcJsMinimizerPluginOptions;
	emotion?: boolean | EmotionConfig;
	presetEnv?: Partial<RawPresetEnv>;
	polyfill?: boolean;
	devFriendlySplitChunks?: boolean;
	copy?: CopyConfig;
	banner?: BannerPluginOptions | BannerPluginOptions[];
	pluginImport?: PluginImportConfig[];
	relay?: boolean | RawRelayConfig;
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
	patterns: (
		| string
		| ({
				from: string;
		  } & Partial<RawPattern>)
	)[];
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

function resolveTreeShaking(
	treeShaking: Builtins["treeShaking"],
	production: boolean
): string {
	return treeShaking !== undefined
		? treeShaking.toString()
		: production
		? "true"
		: "false";
}

function resolveProvide(
	provide: Builtins["provide"] = {}
): Record<string, string[]> {
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

export function deprecated_resolveBuiltins(
	builtins: Builtins,
	options: RspackOptionsNormalized
) {
	const contextPath = options.context!;
	const presetEnv = resolvePresetEnv(builtins.presetEnv, contextPath);
	builtins.presetEnv ?? loadConfig({ path: contextPath }) ?? [];
	options.plugins.push();

	// return {
	// 	css: css
	// 		? {
	// 				modules: {
	// 					localsConvention: "asIs",
	// 					localIdentName: production
	// 						? "[hash]"
	// 						: "[path][name][ext]__[local]",
	// 					exportsOnly: false,
	// 					...builtins.css?.modules
	// 				}
	// 		  }
	// 		: undefined,
	// 	postcss: { pxtorem: undefined, ...builtins.postcss },
	// 	treeShaking: resolveTreeShaking(builtins.treeShaking, production),
	// 	react: builtins.react ?? {},
	// 	noEmitAssets: builtins.noEmitAssets ?? false,
	// 	define: resolveDefine(builtins.define || {}),
	// 	provide: resolveProvide(builtins.provide),
	// 	html: resolveHtml(builtins.html || []),
	// 	presetEnv,
	// 	progress: resolveProgress(builtins.progress),
	// 	decorator: resolveDecorator(builtins.decorator),
	// 	minifyOptions: resolveMinifyOptions(builtins, optimization),
	// 	emotion: resolveEmotion(builtins.emotion, production),
	// 	devFriendlySplitChunks: builtins.devFriendlySplitChunks ?? false,
	// 	copy: resolveCopy(builtins.copy),
	// 	banner: resolveBanner(builtins.banner),
	// 	pluginImport: resolvePluginImport(builtins.pluginImport),
	// 	relay: builtins.relay
	// 		? resolveRelay(builtins.relay, contextPath)
	// 		: undefined,
	// };
}
