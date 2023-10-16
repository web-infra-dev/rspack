import { RawProgressPluginOptions } from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";

export type ProgressPluginArgument = RawProgressPluginOptions | undefined;
export const ProgressPlugin = create(
	BuiltinPluginName.ProgressPlugin,
	(progress: ProgressPluginArgument = {}): RawProgressPluginOptions => progress
);
