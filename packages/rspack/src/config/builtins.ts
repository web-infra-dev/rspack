import type { RawBuiltins, RawHtmlPluginConfig } from "@rspack/binding";
import { loadConfig } from "browserslist";

export type BuiltinsHtmlPluginConfig = Omit<RawHtmlPluginConfig, "meta"> & {
	meta?: Record<string, string | Record<string, string>>;
};

export type Builtins = Omit<RawBuiltins, "browserslist" | "html"> & {
	polyfillBuiltins?: boolean; // polyfill node builtin api
	html?: Array<BuiltinsHtmlPluginConfig>;
};

export type ResolvedBuiltins = Omit<RawBuiltins, "html"> & {
	polyfillBuiltins?: boolean;
	html?: Array<BuiltinsHtmlPluginConfig>;
};

function resolveDefine(define = {}) {
	const entries = Object.entries(define).map(([key, value]) => [
		key,
		JSON.stringify(value)
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

export function resolveBuiltinsOptions(
	builtins: Builtins,
	contextPath: string
): ResolvedBuiltins {
	const browserslist = loadConfig({ path: contextPath }) || [];
	return {
		...builtins,
		html: resolveHtml(builtins.html || []),
		browserslist
	};
}
