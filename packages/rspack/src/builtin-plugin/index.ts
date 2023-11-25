export { RspackBuiltinPlugin } from "./base";

export * from "./DefinePlugin";
export * from "./ProvidePlugin";
export * from "./BannerPlugin";
export * from "./ProgressPlugin";
export * from "./EntryPlugin";
export * from "./ExternalsPlugin";
export * from "./NodeTargetPlugin";
export * from "./ElectronTargetPlugin";
export * from "./HttpExternalsRspackPlugin";
export * from "./EnableChunkLoadingPlugin";
export * from "./EnableLibraryPlugin";
export * from "./EnableWasmLoadingPlugin";
export * from "./ArrayPushCallbackChunkFormatPlugin";
export * from "./CommonJsChunkFormatPlugin";
export * from "./ModuleChunkFormatPlugin";
export * from "./HotModuleReplacementPlugin";
export * from "./WebWorkerTemplatePlugin";
export * from "./LimitChunkCountPlugin";
export * from "./MergeDuplicateChunksPlugin";
export * from "./SplitChunksPlugin";

export * from "./HtmlRspackPlugin";
export * from "./CopyRspackPlugin";
export * from "./SwcJsMinimizerPlugin";
export * from "./SwcCssMinimizerPlugin";

///// DEPRECATED /////
import {
	RawDecoratorOptions,
	RawPresetEnv,
	RawProgressPluginOptions,
	RawBuiltins,
	RawCssModulesConfig
} from "@rspack/binding";
import { termlink, deprecatedWarn } from "../util";
import {
	Compiler,
	CopyRspackPlugin,
	CopyRspackPluginOptions,
	HtmlRspackPlugin,
	HtmlRspackPluginOptions,
	RspackOptionsNormalized,
	SwcCssMinimizerRspackPlugin,
	SwcJsMinimizerRspackPlugin,
	SwcJsMinimizerRspackPluginOptions
} from "..";
import {
	BannerPluginOptions,
	DefinePlugin,
	ProvidePlugin,
	ProgressPlugin,
	BannerPlugin
} from ".";
import { loadConfig } from "browserslist";
import {
	EmotionOptions,
	PluginImportOptions,
	ReactOptions,
	RelayOptions,
	resolveEmotion,
	resolvePluginImport,
	resolveReact,
	resolveRelay
} from "../builtin-loader/swc";

type BuiltinsCssConfig = {
	modules?: Partial<RawCssModulesConfig>;
};

type EmotionConfigImportMap = {
	[packageName: string]: {
		[exportName: string]: {
			canonicalImport?: [string, string];
		};
	};
};

type EmotionConfig = {
	sourceMap?: boolean;
	autoLabel?: "never" | "dev-only" | "always";
	labelFormat?: string;
	importMap?: EmotionConfigImportMap;
};

type PluginImportConfig = {
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

export interface Builtins {
	css?: BuiltinsCssConfig;
	treeShaking?: boolean | "module";
	progress?: boolean | Partial<RawProgressPluginOptions>;
	noEmitAssets?: boolean;
	define?: Record<string, string | boolean | undefined>;
	provide?: Record<string, string | string[]>;
	html?: Array<HtmlRspackPluginOptions>;
	decorator?: boolean | Partial<RawDecoratorOptions>;
	minifyOptions?: SwcJsMinimizerRspackPluginOptions;
	presetEnv?: Partial<RawPresetEnv>;
	devFriendlySplitChunks?: boolean;
	copy?: CopyRspackPluginOptions;
	banner?: BannerPluginOptions | BannerPluginOptions[];
	react?: ReactOptions;
	pluginImport?: PluginImportOptions;
	emotion?: EmotionOptions;
	relay?: RelayOptions;
}

export function deprecated_resolveBuiltins(
	builtins: Builtins,
	options: RspackOptionsNormalized,
	compiler: Compiler
): RawBuiltins {
	// deprecatedWarn(
	// 	`'configuration.builtins' has been deprecated, and will be drop support in 0.6.0, please follow ${termlink(
	// 		"the migration guide",
	// 		"https://www.rspack.dev/en/config/builtins.html" // TODO: write a migration guide
	// 	)}`,
	// 	enableDeprecatedWarning
	// );
	const contextPath = options.context!;
	const production = options.mode === "production" || !options.mode;
	const isRoot = !compiler.isChild();
	if (builtins.define) {
		isRoot &&
			deprecatedWarn(
				`'builtins.define = ${JSON.stringify(
					builtins.define
				)}' has been deprecated, please migrate to ${termlink(
					"rspack.DefinePlugin",
					"https://www.rspack.dev/config/plugins.html#defineplugin"
				)}`
			);
		new DefinePlugin(builtins.define).apply(compiler);
	}
	if (builtins.provide) {
		isRoot &&
			deprecatedWarn(
				`'builtins.provide = ${JSON.stringify(
					builtins.provide
				)}' has been deprecated, please migrate to ${termlink(
					"rspack.ProvidePlugin",
					"https://www.rspack.dev/config/plugins.html#provideplugin"
				)}`
			);
		new ProvidePlugin(builtins.provide).apply(compiler);
	}
	if (builtins.progress) {
		isRoot &&
			deprecatedWarn(
				`'builtins.progress = ${JSON.stringify(
					builtins.progress
				)}' has been deprecated, please migrate to ${termlink(
					"rspack.ProgressPlugin",
					"https://www.rspack.dev/config/plugins.html#progressplugin"
				)}`
			);
		const progress = builtins.progress === true ? {} : builtins.progress;
		new ProgressPlugin(progress).apply(compiler);
	}
	if (builtins.banner) {
		isRoot &&
			deprecatedWarn(
				`'builtins.banner = ${JSON.stringify(
					builtins.banner
				)}' has been deprecated, please migrate to ${termlink(
					"rspack.BannerPlugin",
					"https://www.rspack.dev/config/plugins.html#bannerplugin"
				)}`
			);
		if (Array.isArray(builtins.banner)) {
			for (const banner of builtins.banner) {
				new BannerPlugin(banner).apply(compiler);
			}
		} else {
			new BannerPlugin(builtins.banner).apply(compiler);
		}
	}

	if (builtins.html) {
		isRoot &&
			deprecatedWarn(
				`'builtins.html = ${JSON.stringify(
					builtins.html
				)}' has been deprecated, please migrate to ${termlink(
					"rspack.HtmlRspackPlugin",
					"https://www.rspack.dev/config/plugins.html#htmlrspackplugin"
				)}`
			);
		for (const html of builtins.html) {
			new HtmlRspackPlugin(html).apply(compiler);
		}
	}
	if (builtins.copy) {
		isRoot &&
			deprecatedWarn(
				`'builtins.copy = ${JSON.stringify(
					builtins.copy
				)}' has been deprecated, please migrate to ${termlink(
					"rspack.CopyRspackPlugin",
					"https://www.rspack.dev/config/plugins.html#copyrspackplugin"
				)}`
			);
		new CopyRspackPlugin(builtins.copy).apply(compiler);
	}
	if (builtins.minifyOptions) {
		isRoot &&
			deprecatedWarn(
				`'builtins.minifyOptions = ${JSON.stringify(
					builtins.minifyOptions
				)}' has been deprecated, please migrate to ${termlink(
					"rspack.SwcJsMinimizerRspackPlugin",
					"https://www.rspack.dev/config/plugins.html#SwcJsMinimizerRspackPlugin"
				)} and ${termlink(
					"rspack.SwcCssMinimizerRspackPlugin",
					"https://www.rspack.dev/config/plugins.html#SwcCssMinimizerRspackPlugin"
				)}`
			);
	}
	if (builtins.devFriendlySplitChunks) {
		isRoot &&
			deprecatedWarn(
				`'builtins.devFriendlySplitChunks = ${JSON.stringify(
					builtins.devFriendlySplitChunks
				)}' has been deprecated, please switch to 'builtins.devFriendlySplitChunks = false' to use webpack's behavior.`
			);
	}
	const disableMinify =
		!options.optimization.minimize ||
		options.optimization.minimizer!.some(item => item !== "...");
	if (!disableMinify) {
		new SwcJsMinimizerRspackPlugin(builtins.minifyOptions).apply(compiler);
		new SwcCssMinimizerRspackPlugin().apply(compiler);
	}

	let noEmitAssets = false;
	if (builtins.noEmitAssets) {
		isRoot &&
			deprecatedWarn(
				`'builtins.noEmitAssets = ${JSON.stringify(
					builtins.noEmitAssets
				)}' has been deprecated, this is only a temporary workaround for memory output FS, since Rspack have already supported memory output FS, so you can safely remove this`
			);
		noEmitAssets = true;
	}

	if (options.experiments.rspackFuture?.disableTransformByDefault) {
		(
			[
				"react",
				"pluginImport",
				"decorator",
				"presetEnv",
				"emotion",
				"relay"
			] as const
		).forEach(key => {
			if (builtins[key]) {
				isRoot &&
					deprecatedWarn(
						`'builtins.${key} = ${JSON.stringify(
							builtins[key]
						)}' only works for 'experiments.rspackFuture.disableTransformByDefault = false', please migrate to ${termlink(
							"builtin:swc-loader options",
							"https://www.rspack.dev/guide/loader.html#builtinswc-loader"
						)}`,
						true
					);
			}
		});
	}

	return {
		// TODO: discuss with webpack, this should move to css generator options
		css: options.experiments.css
			? {
					modules: {
						localsConvention: "asIs",
						localIdentName: production
							? "[hash]"
							: "[path][name][ext]__[local]",
						exportsOnly: false,
						...builtins.css?.modules
					}
			  }
			: undefined,
		treeShaking: resolveTreeShaking(builtins.treeShaking, production),
		noEmitAssets: noEmitAssets,
		presetEnv: resolvePresetEnv(builtins.presetEnv, contextPath),
		decorator: resolveDecorator(builtins.decorator),
		devFriendlySplitChunks: builtins.devFriendlySplitChunks ?? false,
		react: resolveReact(builtins.react),
		pluginImport: resolvePluginImport(builtins.pluginImport),
		emotion: builtins.emotion
			? JSON.stringify(resolveEmotion(builtins.emotion, production))
			: undefined,
		relay: resolveRelay(builtins.relay, contextPath)
	};
}
