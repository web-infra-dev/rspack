import { BuiltinPluginName, RawProgressPluginOptions } from "@rspack/binding";

import { create } from "./base";

export type ProgressPluginArgument =
	| Partial<RawProgressPluginOptions>
	| undefined;
export const ProgressPlugin = create(
	BuiltinPluginName.ProgressPlugin,
	(progress: ProgressPluginArgument = {}): RawProgressPluginOptions => ({
		prefix: progress.prefix ?? "",
		profile: progress.profile ?? false,
		template:
			progress.template ??
			"● {prefix:.bold} {bar:25.green/white.dim} ({percent}%) {wide_msg:.dim}",
		tickStrings: progress.tickStrings,
		progressChars: progress.progressChars ?? "━━"
	})
);
