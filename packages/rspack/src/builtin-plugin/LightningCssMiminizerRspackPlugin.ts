import {
	BuiltinPluginName,
	type RawLightningCssMinimizerRspackPluginOptions
} from "@rspack/binding";

import { create } from "./base";

type MinifyCondition = string | RegExp;
type MinifyConditions = MinifyCondition | MinifyCondition[];

export type LightningCssMinimizerRspackPluginOptions =
	Partial<RawLightningCssMinimizerRspackPluginOptions> & {
		test?: MinifyConditions;
		exclude?: MinifyConditions;
		include?: MinifyConditions;
	};

export const LightningCssMinimizerRspackPlugin = create(
	BuiltinPluginName.LightningCssMinimizerRspackPlugin,
	(
		options?: LightningCssMinimizerRspackPluginOptions
	): RawLightningCssMinimizerRspackPluginOptions => {
		return {
			errorRecovery: options?.errorRecovery ?? true,
			unusedSymbols: options?.unusedSymbols ?? [],
			removeUnusedLocalIdents: options?.removeUnusedLocalIdents ?? true,
			browserslist: options?.browserslist ?? ["defaults"],
			test: options?.test,
			include: options?.include,
			exclude: options?.exclude
		};
	}
);
