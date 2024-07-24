import {
	BuiltinPluginName,
	type RawSwcCssMinimizerRspackPluginOptions
} from "@rspack/binding";

import { create } from "./base";

type MinifyCondition = string | RegExp;
type MinifyConditions = MinifyCondition | MinifyCondition[];

export type SwcCssMinimizerRspackPluginOptions = {
	test?: MinifyConditions;
	exclude?: MinifyConditions;
	include?: MinifyConditions;
};

export const SwcCssMinimizerRspackPlugin = create(
	BuiltinPluginName.SwcCssMinimizerRspackPlugin,
	(
		options?: SwcCssMinimizerRspackPluginOptions
	): RawSwcCssMinimizerRspackPluginOptions => {
		return {
			test: options?.test,
			include: options?.include,
			exclude: options?.exclude
		};
	}
);
