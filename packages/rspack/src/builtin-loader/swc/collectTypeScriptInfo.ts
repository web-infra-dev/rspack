import * as z from "zod/v4";

export type CollectTypeScriptInfoOptions = {
	typeExports?: boolean;
};

export const ZodSwcCollectTypeScriptInfo = z.strictObject({
	typeExports: z.boolean().optional()
}) satisfies z.ZodType<CollectTypeScriptInfoOptions>;
