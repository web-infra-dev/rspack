export type CollectTypeScriptInfoOptions = {
	typeExports?: boolean;
	exportedEnum?: boolean | "const-only";
};

export function resolveCollectTypeScriptInfo(
	options: CollectTypeScriptInfoOptions
) {
	return {
		typeExports: options.typeExports,
		exportedEnum:
			options.exportedEnum === true
				? "all"
				: options.exportedEnum === false
					? "none"
					: "const-only"
	};
}
