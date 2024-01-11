import { RawBundlerInfoPluginOptions } from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";

export type BundleInfoOptions = {
	version?: string;
	mode?: "auto" | "all" | string[];
};

export const BundlerInfoPlugin = create(
	BuiltinPluginName.BundlerInfoPlugin,
	(options: BundleInfoOptions): RawBundlerInfoPluginOptions => {
		return {
			version: options.version || "unknown",
			mode: options.mode || "auto"
		};
	}
);
