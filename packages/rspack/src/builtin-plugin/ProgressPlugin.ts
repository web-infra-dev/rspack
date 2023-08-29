import { RawProgressPluginConfig } from "@rspack/binding";
import { BuiltinPluginKind, create } from "./base";

export type ProgressPluginOptions = RawProgressPluginConfig | undefined;
export const ProgressPlugin = create(
	BuiltinPluginKind.Progress,
	(progress: ProgressPluginOptions = {}): RawProgressPluginConfig => progress
);
