import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

type MinifyCondition = string | RegExp;
type MinifyConditions = MinifyCondition | MinifyCondition[];

export type SwcCssMinimizerRspackPluginOptions = {
	exclude?: MinifyConditions;
	include?: MinifyConditions;
};

export const SwcCssMinimizerRspackPlugin = create(
	BuiltinPluginName.SwcCssMinimizerRspackPlugin,
	(options?: SwcCssMinimizerRspackPluginOptions) => undefined
);
