import {
	BuiltinPluginName,
	RawBundlerInfoPluginOptions
} from "@rspack/binding";
import { create } from "./base";

export type BundleInfoOptions = {
	version?: string;
	force?: boolean | string[];
};

export const BundlerInfoRspackPlugin = create(
	BuiltinPluginName.BundlerInfoRspackPlugin,
	(options: BundleInfoOptions): RawBundlerInfoPluginOptions => {
		return {
			version: options.version || "unknown",
			force: options.force ?? false
		};
	}
);
