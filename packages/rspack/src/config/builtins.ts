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
	RawMinificationConditions,
	RawCssExperimentOptions
} from "@rspack/binding";
import { loadConfig } from "browserslist";
import { Optimization, RspackOptions, RspackOptionsNormalized } from "..";
import {
	BannerPluginOptions,
	NoEmitAssetsPlugin,
	SwcJsMinimizerPluginOptions,
	TreeShakingPlugin
} from "../builtin-plugin";
import {
	EmotionConfig,
	PluginImportConfig,
	ReactOptionsPlugin
} from "../builtin-plugin/deprecate-by-swc-loader";
import { CopyPluginOptions } from "../builtin-plugin/copy";
import { HtmlPluginOptions } from "../builtin-plugin/html";
import { termlink, deprecatedWarn } from "../util";

export type BuiltinsCssConfig = {
	modules?: Partial<RawCssExperimentOptions>;
};

export interface Builtins {
	css?: BuiltinsCssConfig;
	treeShaking?: boolean | "module";
	progress?: boolean | RawProgressPluginConfig;
	react?: RawReactOptions;
	noEmitAssets?: boolean;
	define?: Record<string, string | boolean | undefined>;
	provide?: Record<string, string | string[]>;
	html?: Array<HtmlPluginOptions>;
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

export function deprecated_resolveBuiltins(
	builtins: Builtins | undefined,
	options: RspackOptionsNormalized
) {
	if (!builtins) return;
	const defaultEnableDeprecatedWarning = false; // enable this when all prepare is ready
	const enableDeprecatedWarning =
		(process.env.RSPACK_BUILTINS_DEPRECATED ??
			`${defaultEnableDeprecatedWarning}`) !== "false";
	deprecatedWarn(
		`configuration.builtins has been deprecated, and will be drop support in 0.6.0, please follow ${termlink(
			"the migration guide",
			"https://www.rspack.dev/en/config/builtins.html"
		)}`,
		enableDeprecatedWarning
	);
	const contextPath = options.context!;
	const production = options.mode === "production" || !options.mode;
	if (builtins.css) {
		deprecatedWarn(
			`You are still using builtins.css, please migrate to experiments.css`,
			enableDeprecatedWarning
		);
		options.experiments.css = {
			localsConvention: "asIs",
			localIdentName: production ? "[hash]" : "[path][name][ext]__[local]",
			exportsOnly: false,
			...builtins.css?.modules
		};
	}
	if (builtins.treeShaking) {
		// TODO: wait tree shaking refactor
		// deprecatedWarn(`You are still using builtins.treeShaking = ${JSON.stringify(builtins.treeShaking)}, please migrate to ...`, enableDeprecatedWarning)
		options.plugins.push(
			new TreeShakingPlugin({ production, enable: builtins.treeShaking })
		);
	}
	// TODO: wait builtin:swc-loader
	// deprecatedWarn(`You are still using builtins.react = ${JSON.stringify(builtins.react)}, please migrate to builtin:swc-loader`, enableDeprecatedWarning)
	options.plugins.push(new ReactOptionsPlugin(builtins.react));
	if (builtins.noEmitAssets) {
		options.plugins.push(new NoEmitAssetsPlugin(undefined));
	}
	// return {
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
