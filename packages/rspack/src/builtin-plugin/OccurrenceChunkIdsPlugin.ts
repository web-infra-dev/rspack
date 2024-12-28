import {
	BuiltinPluginName,
	type RawOccurrenceChunkIdsPluginOptions
} from "@rspack/binding";

import { create } from "./base";

export const OccurrenceChunkIdsPlugin = create(
	BuiltinPluginName.OccurrenceChunkIdsPlugin,
	(options?: RawOccurrenceChunkIdsPluginOptions) => ({ ...options }),
	"compilation"
);
