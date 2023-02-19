import type {
	RawBuiltins,
	RawHtmlPluginConfig,
	RawDecoratorOptions,
	RawMinification,
	RawReactOptions,
	RawProgressPluginConfig,
	RawPostCssConfig,
	RawCssPluginConfig
} from "@rspack/binding";
import { loadConfig } from "browserslist";

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

export interface Builtins {
	css?: RawCssPluginConfig;
	postcss?: RawPostCssConfig;
	treeShaking?: boolean;
	progress?: boolean | RawProgressPluginConfig;
	react?: RawReactOptions;
	noEmitAssets?: boolean;
	define?: Record<string, string | undefined>;
	html?: Array<BuiltinsHtmlPluginConfig>;
	decorator?: boolean | Partial<RawDecoratorOptions>;
	minify?: boolean | Partial<RawMinification>;
	emotion?: EmotionConfig;
	browserslist?: string[];
	polyfill?: boolean;
	devFriendlySplitChunks?: boolean;
}

export type ResolvedBuiltins = Omit<RawBuiltins, "html"> & {
	html?: Array<BuiltinsHtmlPluginConfig>;
	emotion?: string;
};

function resolveDefine(define: Builtins["define"]): RawBuiltins["define"] {
	// @ts-expect-error
	const entries = Object.entries(define).map(([key, value]) => [
		key,
		value === undefined ? "undefined" : value
	]);
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

export function resolveBuiltinsOptions(
	builtins: Builtins,
	{
		contextPath,
		production,
		development
	}: { contextPath: string; production: boolean; development: boolean }
): RawBuiltins {
	const browserslist =
		builtins.browserslist ?? loadConfig({ path: contextPath }) ?? [];
	return {
		css: {
			presetEnv: builtins.css?.presetEnv ? builtins.css.presetEnv : [],
			modules: {
				localsConvention: "asIs",
				localIdentName: production ? "[hash]" : "[path][name][ext]__[local]",
				exportsOnly: false,
				...builtins.css?.modules
			}
		},
		postcss: { pxtorem: undefined, ...builtins.postcss },
		treeShaking: builtins.treeShaking ?? production ? true : false,
		progress: builtins.progress
			? { prefix: undefined, ...builtins.progress }
			: undefined,
		react: builtins.react ?? {},
		noEmitAssets: builtins.noEmitAssets ?? false,
		define: resolveDefine(builtins.define || {}),
		html: resolveHtml(builtins.html || []),
		browserslist,
		progress: resolveProgress(builtins.progress),
		decorator: resolveDecorator(builtins.decorator),
		minify: resolveMinify(builtins, production),
		emotion: resolveEmotion(builtins.emotion, production),
		polyfill: builtins.polyfill ?? true,
		devFriendlySplitChunks: builtins.devFriendlySplitChunks ?? false
	};
}

export function resolveMinify(
	builtins: Builtins,
	isProduction: boolean
): RawMinification {
	if (builtins.minify) {
		if (builtins.minify === true) {
			return {
				enable: true,
				passes: 1,
				dropConsole: false,
				pureFuncs: []
			};
		} else {
			return {
				enable: true,
				passes: 1,
				dropConsole: false,
				pureFuncs: [],
				...builtins.minify
			};
		}
	} else if (builtins.minify === false) {
		return {
			enable: false,
			passes: 1,
			dropConsole: false,
			pureFuncs: []
		};
	} else {
		return {
			enable: isProduction,
			passes: 1,
			dropConsole: false,
			pureFuncs: []
		};
	}
}
