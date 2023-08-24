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
import {
	EmotionConfig,
	PluginImportConfig
} from "../builtin-plugin/depracate-by-swc-loader";
import { CopyPluginOptions } from "../builtin-plugin/copy";

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
	devFriendlySplitChunks?: boolean;
	copy?: CopyPluginOptions;
	banner?: BannerPluginOptions | BannerPluginOptions[];
	pluginImport?: PluginImportConfig[];
	relay?: boolean | RawRelayConfig;
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

export function deprecated_resolveBuiltins(
	builtins: Builtins,
	options: RspackOptionsNormalized
) {
	const contextPath = options.context!;
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
