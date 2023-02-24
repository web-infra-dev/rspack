import type {
	RawBuiltins,
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
	RawCssModulesConfig
} from "@rspack/binding";
import { loadConfig } from "browserslist";
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

export interface Builtins {
	css?: CssPluginConfig;
	postcss?: RawPostCssConfig;
	treeShaking?: boolean;
	progress?: boolean | RawProgressPluginConfig;
	react?: RawReactOptions;
	noEmitAssets?: boolean;
	define?: Record<string, string | boolean | undefined>;
	html?: Array<BuiltinsHtmlPluginConfig>;
	decorator?: boolean | Partial<RawDecoratorOptions>;
	minifyOptions?: Partial<RawMinification>;
	emotion?: EmotionConfig;
	presetEnv?: Partial<RawBuiltins["presetEnv"]>;
	polyfill?: boolean;
	devFriendlySplitChunks?: boolean;
	copy?: CopyConfig;
	pluginImport?: PluginImportConfig[];
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
		treeShaking: builtins.treeShaking ?? production ? true : false,
		react: builtins.react ?? {},
		noEmitAssets: builtins.noEmitAssets ?? false,
		define: resolveDefine(builtins.define || {}),
		html: resolveHtml(builtins.html || []),
		presetEnv,
		progress: resolveProgress(builtins.progress),
		decorator: resolveDecorator(builtins.decorator),
		minifyOptions: resolveMinifyOptions(builtins, optimization),
		emotion: resolveEmotion(builtins.emotion, production),
		devFriendlySplitChunks: builtins.devFriendlySplitChunks ?? false,
		copy: resolveCopy(builtins.copy),
		pluginImport: resolvePluginImport(builtins.pluginImport)
	};
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

	return {
		passes: 1,
		dropConsole: false,
		pureFuncs: [],
		...builtins.minifyOptions
	};
}
