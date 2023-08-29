import { RawCopyConfig, RawPattern } from "@rspack/binding";
import { BuiltinPluginKind, create } from "./base";

export type CopyPluginOptions = {
	patterns: (
		| string
		| ({
				from: string;
		  } & Partial<RawPattern>)
	)[];
};

export const CopyPlugin = create(
	BuiltinPluginKind.Copy,
	(copy: CopyPluginOptions): RawCopyConfig => {
		const ret: RawCopyConfig = {
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

			return pattern as RawPattern;
		});

		return ret;
	}
);
