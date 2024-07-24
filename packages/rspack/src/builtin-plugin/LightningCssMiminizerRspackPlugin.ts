import {
	BuiltinPluginName,
	type RawLightningCssMinimizerRspackPluginOptions
} from "@rspack/binding";

import { create } from "./base";

export type LightningCssMinimizerRspackPluginOptions =
	Partial<RawLightningCssMinimizerRspackPluginOptions>;

export const LightningCssMinimizerRspackPlugin = create(
	BuiltinPluginName.LightningCssMinimizerRspackPlugin,
	(
		options?: LightningCssMinimizerRspackPluginOptions
	): RawLightningCssMinimizerRspackPluginOptions => {
		return {
			errorRecovery: options?.errorRecovery ?? true,
			unusedSymbols: options?.unusedSymbols ?? [],
			removeUnusedLocalIdents: options?.removeUnusedLocalIdents ?? true,
			browserslist: options?.browserslist ?? ["defaults"]
		};
	}
);
