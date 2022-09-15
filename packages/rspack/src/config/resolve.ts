export type Resolve = {
	preferRelative?: boolean;
	extensions?: string[];
	mainFiles?: string[];
	mainFields?: string[];
	browserField?: boolean;
	conditionNames?: string[];
};

export type ResolvedResolve = {
	preferRelative: boolean;
	extensions: string[];
	mainFiles: string[];
	mainFields: string[];
	browserField: boolean;
	conditionNames: string[];
};

export function resolveResolveOptions(resolve: Resolve = {}): ResolvedResolve {
	const preferRelative = resolve.preferRelative ?? false;
	const extensions = resolve.extensions ?? [
		".tsx",
		".jsx",
		".ts",
		".js",
		".json",
		".d.ts"
	];
	const mainFields = resolve.mainFields ?? ["module", "main"];
	const mainFiles = resolve.mainFiles ?? ["index"];
	const browserField = resolve.browserField ?? true;
	const conditionNames = resolve.conditionNames ?? ["module", "import"];
	return {
		preferRelative,
		extensions,
		mainFiles,
		mainFields,
		browserField,
		conditionNames
	};
}
