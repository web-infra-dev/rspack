import {
	BuiltinPluginName,
	type RawRuntimeChunkOptions
} from "@rspack/binding";

import { create } from "./base";

export type RuntimeChunkPluginOptions = RawRuntimeChunkOptions;

export const RuntimeChunkPlugin = create(
	BuiltinPluginName.RuntimeChunkPlugin,
	(options: RuntimeChunkPluginOptions): RawRuntimeChunkOptions => options,
	"thisCompilation"
);
