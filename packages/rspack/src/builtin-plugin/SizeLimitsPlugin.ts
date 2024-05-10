import { BuiltinPluginName, RawSizeLimitsPluginOptions } from "@rspack/binding";
import { create } from "./base";
import { Performance } from "..";

export const SizeLimitsPlugin = create(
	BuiltinPluginName.SizeLimitsPlugin,
	(options: Exclude<Performance, false>): RawSizeLimitsPluginOptions => {
		const hints = options.hints === false ? undefined : options.hints;

		return { ...options, hints };
	}
);
