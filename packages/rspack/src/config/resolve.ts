import type { ResolvedTarget } from "./target";

export type Resolve = {
	preferRelative?: boolean;
	extensions?: string[];
	mainFiles?: string[];
	mainFields?: string[];
	browserField?: boolean;
	conditionNames?: string[];
	alias?: Record<string, string>;
	tsConfigPath?: string;
};

export type ResolvedResolve = {
	preferRelative: boolean;
	extensions: string[];
	mainFiles: string[];
	mainFields: string[];
	browserField: boolean;
	conditionNames: string[];
	alias: Record<string, string>;
	tsConfigPath: string;
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
	const conditionNames = resolve.conditionNames ?? ["module", "import"];
	const tsConfigPath = resolve.tsConfigPath ?? "";
	return {
		preferRelative,
		extensions,
		mainFiles,
		mainFields,
		browserField,
		conditionNames,
		alias,
		tsConfigPath
	};
}
