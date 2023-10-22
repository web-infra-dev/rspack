import { RawLimitChunkCountPluginOptions } from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";

export type LimitChunkCountOptions = {};

export const LimitChunkCountPlugin = create(
	BuiltinPluginName.LimitChunkCountPlugin,
	(): RawLimitChunkCountPluginOptions => {
		return {};
	}
);
