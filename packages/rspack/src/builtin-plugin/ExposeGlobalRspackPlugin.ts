import {
	BuiltinPluginName,
	RawExposeGlobalPluginOptions
} from "@rspack/binding";

import { create } from "./base";

export const ExposeGlobalRspackPlugin = create(
	BuiltinPluginName.ExposeGlobalRspackPlugin,
	(options: RawExposeGlobalPluginOptions): RawExposeGlobalPluginOptions =>
		options
);
