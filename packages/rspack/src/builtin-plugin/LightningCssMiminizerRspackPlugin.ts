import {
	BuiltinPluginName,
	RawLightningCssMinimizerRspackPluginOptions
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
			errorRecovery: options?.errorRecovery ?? false,
			unusedSymbols: options?.unusedSymbols ?? ["..."],
			browserslist: options?.browserslist ?? ["defaults"]
		};
	}
);
