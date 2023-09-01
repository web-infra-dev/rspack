export { RspackBuiltinPlugin } from "./base";

export * from "./DefinePlugin";
export * from "./ProvidePlugin";
export * from "./BannerPlugin";
export * from "./ProgressPlugin";

export * from "./HtmlPlugin";
export * from "./CopyPlugin";
export * from "./SwcJsMinimizerPlugin";
export * from "./SwcCssMinimizerPlugin";

///// DEPRECATED /////
import {
	RawDecoratorOptions,
	RawPresetEnv,
	RawProgressPluginConfig,
	RawReactOptions,
	RawRelayConfig,
	RawPluginImportConfig,
	RawBuiltins,
	RawCssModulesConfig
} from "@rspack/binding";
import { deprecatedWarn } from "../util";
import { Compiler, RspackOptionsNormalized } from "..";
import {
	HtmlPluginOptions,
	SwcJsMinimizerPluginOptions,
	CopyPluginOptions,
	BannerPluginOptions,
	DefinePlugin,
	ProvidePlugin,
	ProgressPlugin,
	HtmlPlugin,
	CopyPlugin,
	BannerPlugin,
	SwcJsMinimizerPlugin,
	SwcCssMinimizerPlugin
} from ".";
import { loadConfig } from "browserslist";
import path from "path";

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

type RelayConfig = boolean | RawRelayConfig;

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
	builtins: Builtins,
	options: RspackOptionsNormalized,
	compiler: Compiler
): RawBuiltins {
	const defaultEnableDeprecatedWarning = false;
	const enableDeprecatedWarning =
		(process.env.RSPACK_BUILTINS_DEPRECATED ??
			`${defaultEnableDeprecatedWarning}`) !== "false";
	// deprecatedWarn(
	// 	`'configuration.builtins' has been deprecated, and will be drop support in 0.6.0, please follow ${termlink(
	// 		"the migration guide",
	// 		"https://www.rspack.dev/en/config/builtins.html" // TODO: write a migration guide
	// 	)}`,
	// 	enableDeprecatedWarning
	// );
	const contextPath = options.context!;
	const production = options.mode === "production" || !options.mode;
	if (builtins.define) {
		deprecatedWarn(
			`'builtins.define = ${JSON.stringify(
				builtins.define
			)}' has been deprecated, please migrate to rspack.DefinePlugin`,
			enableDeprecatedWarning
		);
		new DefinePlugin(builtins.define).apply(compiler);
	}
	if (builtins.provide) {
		deprecatedWarn(
			`'builtins.provide = ${JSON.stringify(
				builtins.provide
			)}' has been deprecated, please migrate to rspack.ProvidePlugin`,
			enableDeprecatedWarning
		);
		new ProvidePlugin(builtins.provide).apply(compiler);
	}
	if (builtins.progress) {
		deprecatedWarn(
			`'builtins.progress = ${JSON.stringify(
				builtins.progress
			)}' has been deprecated, please migrate to rspack.ProgressPlugin`,
			enableDeprecatedWarning
		);
		const progress = builtins.progress === true ? {} : builtins.progress;
		new ProgressPlugin(progress).apply(compiler);
	}
	if (builtins.banner) {
		deprecatedWarn(
			`'builtins.banner = ${JSON.stringify(
				builtins.banner
			)}' has been deprecated, please migrate to rspack.BannerPlugin`,
			enableDeprecatedWarning
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
		deprecatedWarn(
			`'builtins.html = ${JSON.stringify(
				builtins.html
			)}' has been deprecated, please migrate to rspack.HtmlPlugin`,
			enableDeprecatedWarning
		);
		for (const html of builtins.html) {
			new HtmlPlugin(html).apply(compiler);
		}
	}
	if (builtins.copy) {
		deprecatedWarn(
			`'builtins.copy = ${JSON.stringify(
				builtins.copy
			)}' has been deprecated, please migrate to rspack.CopyPlugin`,
			enableDeprecatedWarning
		);
		new CopyPlugin(builtins.copy).apply(compiler);
	}
	if (builtins.minifyOptions) {
		deprecatedWarn(
			`'builtins.minifyOptions = ${JSON.stringify(
				builtins.minifyOptions
			)}' has been deprecated, please migrate to rspack.SwcJsMinimizerPlugin and rspack.SwcCssMinimizerPlugin`,
			enableDeprecatedWarning
		);
	}
	const disableMinify =
		!options.optimization.minimize ||
		options.optimization.minimizer!.some(item => item !== "...");
	if (!disableMinify) {
		new SwcJsMinimizerPlugin(builtins.minifyOptions).apply(compiler);
		new SwcCssMinimizerPlugin().apply(compiler);
	}

	let noEmitAssets = false;
	if (builtins.noEmitAssets) {
		deprecatedWarn(
			`'builtins.noEmitAssets = ${JSON.stringify(
				builtins.noEmitAssets
			)}' has been deprecated, this is only a temporary workaround for memory output FS, since Rspack have already supported memory output FS, so you can safely remove this`,
			enableDeprecatedWarning
		);
		noEmitAssets = true;
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
		react: builtins.react ?? {},
		noEmitAssets: noEmitAssets,
		presetEnv: resolvePresetEnv(builtins.presetEnv, contextPath),
		decorator: resolveDecorator(builtins.decorator),
		emotion: resolveEmotion(builtins.emotion, production),
		devFriendlySplitChunks: builtins.devFriendlySplitChunks ?? false,
		pluginImport: resolvePluginImport(builtins.pluginImport),
		relay: builtins.relay
			? resolveRelay(builtins.relay, contextPath)
			: undefined
	};
}
