import type { ResolvedTarget } from "./target";

export type Resolve = {
	preferRelative?: boolean;
	extensions?: string[];
	mainFiles?: string[];
	mainFields?: string[];
	browserField?: boolean;
	conditionNames?: string[];
	alias?: Record<string, string | false>;
	tsConfigPath?: string;
	modules?: string | string[];
	fallback?: Record<string, string | false>;
};

export type ResolvedResolve = {
	preferRelative: boolean;
	extensions: string[];
	mainFiles: string[];
	mainFields: string[];
	browserField: boolean;
	conditionNames: string[];
	alias: Record<string, string | false>;
	fallback: Record<string, string | false>;
	tsConfigPath: string | undefined;
	modules: string[];
};

interface ResolveContext {
	target: ResolvedTarget;
}

export function resolveResolveOptions(
	resolve: Resolve = {},
	{ target }: ResolveContext
): ResolvedResolve {
	const preferRelative = resolve.preferRelative ?? false;
	const extensions = resolve.extensions ?? [
		".tsx",
		".jsx",
		".ts",
		".js",
		".json",
		".d.ts"
	];
	const defaultMainFields = ["module", "main"];
	if (target.includes("web")) {
		defaultMainFields.unshift("browser");
	}
	const mainFields = resolve.mainFields ?? defaultMainFields;
	const mainFiles = resolve.mainFiles ?? ["index"];
	const browserField = resolve.browserField ?? true;
	const alias = resolve.alias ?? {};
	const fallback = resolve.fallback ?? {};
	const conditionNames = resolve.conditionNames;
	const tsConfigPath = resolve.tsConfigPath;
	let modules: string[];
	if (typeof resolve.modules === "undefined") {
		modules = ["node_modules"];
	} else if (!Array.isArray(resolve.modules)) {
		modules = [resolve.modules];
	} else {
		modules = resolve.modules;
	}
	return {
		modules,
		preferRelative,
		extensions,
		mainFiles,
		mainFields,
		browserField,
		// @ts-expect-error
		conditionNames,
		alias,
		tsConfigPath,
		fallback
	};
}
