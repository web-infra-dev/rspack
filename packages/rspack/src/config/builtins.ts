import type {
	RawBuiltins,
	RawHtmlPluginConfig,
	RawDecoratorOptions
} from "@rspack/binding";
import { loadConfig } from "browserslist";
import { Dev } from "./devServer";

export type BuiltinsHtmlPluginConfig = Omit<RawHtmlPluginConfig, "meta"> & {
	meta?: Record<string, string | Record<string, string>>;
};

export type Builtins = Omit<
	RawBuiltins,
	"define" | "browserslist" | "html" | "decorator"
> & {
	define?: Record<string, string | undefined>;
	polyfillBuiltins?: boolean; // polyfill node builtin api
	html?: Array<BuiltinsHtmlPluginConfig>;
	decorator?: boolean | Partial<RawDecoratorOptions>;
};

export type ResolvedBuiltins = Omit<RawBuiltins, "html"> & {
	polyfillBuiltins?: boolean;
	html?: Array<BuiltinsHtmlPluginConfig>;
};

function resolveDefine(define: Builtins["define"]): RawBuiltins["define"] {
	const entries = Object.entries(define).map(([key, value]) => [
		key,
		value === undefined ? "undefined" : value
	]);
	return Object.fromEntries(entries);
}

function resolveHtml(html: Builtins["html"]): BuiltinsHtmlPluginConfig[] {
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
			emitMetadata: true,
			useDefineForClassFields: true
		},
		decorator
	);
}

export function resolveBuiltinsOptions(
	builtins: Builtins,
	contextPath: string
): ResolvedBuiltins {
	const browserslist = loadConfig({ path: contextPath }) || [];
	return {
		...builtins,
		define: resolveDefine(builtins.define || {}),
		html: resolveHtml(builtins.html || []),
		browserslist,
		decorator: resolveDecorator(builtins.decorator)
	};
}
