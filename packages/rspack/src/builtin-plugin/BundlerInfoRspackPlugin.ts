import {
	BuiltinPluginName,
	RawBundlerInfoPluginOptions
} from "@rspack/binding";

import { create } from "./base";

export const BundlerInfoRspackPlugin = create(
	BuiltinPluginName.BundlerInfoRspackPlugin,
	(options: RawBundlerInfoPluginOptions): RawBundlerInfoPluginOptions => options
);
