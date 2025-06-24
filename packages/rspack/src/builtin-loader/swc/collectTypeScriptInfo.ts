import * as z from "zod/v4";

export type CollectTypeScriptInfoOptions = {
	typeExports?: boolean;
	crossModuleEnums?: boolean | "const-only";
};

export const ZodSwcCollectTypeScriptInfo = z.strictObject({
	typeExports: z.boolean().optional(),
	crossModuleEnums: z.boolean().or(z.literal("const-only")).optional()
}) satisfies z.ZodType<CollectTypeScriptInfoOptions>;

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
