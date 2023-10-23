import { RawLimitChunkCountPluginOptions } from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";

export type LimitChunkCountOptions = {
	chunkOverhead?: number;
	entryChunkMultiplicator?: number;
	maxChunks: number;
};

export const LimitChunkCountPlugin = create(
	BuiltinPluginName.LimitChunkCountPlugin,
	(options: LimitChunkCountOptions): RawLimitChunkCountPluginOptions => {
		return options;
	}
);
