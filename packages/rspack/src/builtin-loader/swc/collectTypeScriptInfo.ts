export type CollectTypeScriptInfoOptions = {
	typeExports?: boolean;
	crossModuleEnums?: boolean | "const-only";
};

export function resolveCollectTypeScriptInfo(
	options: CollectTypeScriptInfoOptions
) {
	return {
		typeExports: options.typeExports,
		crossModuleEnums:
			options.crossModuleEnums === true
				? "all"
				: options.crossModuleEnums === false
					? "none"
					: "const-only"
	};
}
