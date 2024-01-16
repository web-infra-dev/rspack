import { RawBundlerInfoPluginOptions } from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";

export type BundleInfoOptions = {
	version?: string;
	force?: boolean | string[];
};

export const BundlerInfoPlugin = create(
	BuiltinPluginName.BundlerInfoPlugin,
	(options: BundleInfoOptions): RawBundlerInfoPluginOptions => {
		return {
			version: options.version || "unknown",
			force: options.force ?? false
		};
	}
);
