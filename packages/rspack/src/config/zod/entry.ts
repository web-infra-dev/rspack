import { z } from "zod";
import { filename, publicPath } from "./output";

const entryItem = z
	.string()
	.min(1)
	.describe("The string is resolved to a module which is loaded upon startup.")
	.or(z.string().min(1).array().min(1));

const entryDescription = z
	.object({
		import: entryItem,
		runtime: z.literal(false).or(z.string().min(1)).optional(),
		publicPath: publicPath().optional(),
		baseUri: z.string().optional(),
		chunkLoading: z
			.literal(false)
			.or(
				z
					.enum(["jsonp", "require", "async-node", "import", "import-scripts"])
					.or(z.string())
					.optional()
			),
		asyncChunks: z.boolean().optional(),
		wasmLoading: z
			.literal(false)
			.or(z.enum(["fetch-streaming", "fetch", "async-node"]))
			.optional(),
		filename: filename().optional()
	})
	.strict();

const entryObject = z.record(entryItem.or(entryDescription));

export function entry() {
	return entryItem.or(entryObject);
}
