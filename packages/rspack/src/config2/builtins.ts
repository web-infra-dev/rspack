import type {
	RawBuiltins,
	RawHtmlPluginConfig,
	RawDecoratorOptions,
	RawMinification
} from "@rspack/binding";
import { loadConfig } from "browserslist";

export type BuiltinsHtmlPluginConfig = Omit<RawHtmlPluginConfig, "meta"> & {
	meta?: Record<string, string | Record<string, string>>;
};

export type ImportMap = {
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
			importMap?: ImportMap;
	  };

export type Builtins = Omit<
	RawBuiltins,
	"define" | "browserslist" | "html" | "decorator" | "minify" | "emotion"
> & {
	define?: Record<string, string | undefined>;
	polyfillBuiltins?: boolean; // polyfill node builtin api
	html?: Array<BuiltinsHtmlPluginConfig>;
	decorator?: boolean | Partial<RawDecoratorOptions>;
	minify?: boolean | Partial<RawMinification>;
	emotion?: EmotionConfig;
};

export type ResolvedBuiltins = Omit<RawBuiltins, "html"> & {
	polyfillBuiltins?: boolean;
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

function resolveHtml(html: Builtins["html"]): BuiltinsHtmlPluginConfig[] {
	// @ts-expect-error
	return html.map(c => {
		for (const key in c.meta) {
			const value = c.meta[key];
			if (typeof value === "string") {
				c.meta[key] = {
					name: key,
					content: value
				};
			}
		}
		return c;
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
	{ contextPath, isProduction }: { contextPath: string; isProduction: boolean }
): ResolvedBuiltins {
	const browserslist = loadConfig({ path: contextPath });
	return {
		...builtins,
		define: resolveDefine(builtins.define || {}),
		html: resolveHtml(builtins.html || []),
		browserslist,
		decorator: resolveDecorator(builtins.decorator),
		minify: resolveMinify(builtins, isProduction),
		emotion: resolveEmotion(builtins.emotion, isProduction)
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
				passes: 1
			};
		} else {
			return {
				...builtins.minify,
				enable: true
			};
		}
	} else if (builtins.minify === false) {
		return {
			enable: false,
			passes: 1
		};
	} else {
		return {
			enable: isProduction,
			passes: 1
		};
	}
}
