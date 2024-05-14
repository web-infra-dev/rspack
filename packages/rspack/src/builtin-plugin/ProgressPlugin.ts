import { BuiltinPluginName, RawProgressPluginOptions } from "@rspack/binding";

import { create } from "./base";

export type ProgressPluginArgument =
	| Partial<RawProgressPluginOptions>
	| undefined;
export const ProgressPlugin = create(
	BuiltinPluginName.ProgressPlugin,
	(progress: ProgressPluginArgument = {}): RawProgressPluginOptions => ({
		prefix: progress.prefix ?? "",
		profile: progress.profile ?? false
	})
);
