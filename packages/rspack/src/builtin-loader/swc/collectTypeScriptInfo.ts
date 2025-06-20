import { z } from "zod";

export type CollectTypeScriptInfoOptions = {
	typeExports?: boolean;
};

export const ZodSwcCollectTypeScriptInfo = z.strictObject({
	typeExports: z.boolean().optional()
}) satisfies z.ZodType<CollectTypeScriptInfoOptions>;
