import { RawProgressPluginConfig } from "@rspack/binding";
import { BuiltinPluginKind, create } from "./base";

export type ProgressPluginArgument = RawProgressPluginConfig | undefined;
export const ProgressPlugin = create(
	BuiltinPluginKind.Progress,
	(progress: ProgressPluginArgument = {}): RawProgressPluginConfig => progress
);
