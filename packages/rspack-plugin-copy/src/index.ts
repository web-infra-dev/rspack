import type { RawCopyConfig } from "@rspack/binding";
import type { Compiler, Plugin } from "@rspack/core";

export type CopyRspackPluginOptions = {
	patterns: string[] | RawCopyConfig["patterns"];
};

export default class RspackCopyPlugin implements Plugin {
	options: CopyRspackPluginOptions;
	constructor(options: CopyRspackPluginOptions = { patterns: [] }) {
		this.options = options;
	}

	apply(compiler: Compiler): void {
		compiler.options.builtins ??= {};
		compiler.options.builtins.copy = resolveCopy(this.options);
	}
}

export function resolveCopy(
	copy?: CopyRspackPluginOptions
): RawCopyConfig | undefined {
	if (!copy) {
		return undefined;
	}

	return {
		patterns: copy.patterns.map(pattern => {
			return typeof pattern === "string" ? { from: pattern } : pattern;
		})
	};
}
