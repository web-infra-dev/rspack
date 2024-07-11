import {
	BuiltinPluginName,
	type RawCopyPattern,
	type RawCopyRspackPluginOptions
} from "@rspack/binding";

import { create } from "./base";

export type CopyRspackPluginOptions = {
	patterns: (
		| string
		| ({
				from: string;
		  } & Partial<RawCopyPattern>)
	)[];
};

export const CopyRspackPlugin = create(
	BuiltinPluginName.CopyRspackPlugin,
	(copy: CopyRspackPluginOptions): RawCopyRspackPluginOptions => {
		const ret: RawCopyRspackPluginOptions = {
			patterns: []
		};

		ret.patterns = (copy.patterns || []).map(pattern => {
			if (typeof pattern === "string") {
				pattern = { from: pattern };
			}

			pattern.force ??= false;
			pattern.noErrorOnMissing ??= false;
			pattern.priority ??= 0;
			pattern.globOptions ??= {};

			return pattern as RawCopyPattern;
		});

		return ret;
	}
);
