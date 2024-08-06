import {
	BuiltinPluginName,
	type RawSwcCssMinimizerRspackPluginOptions
} from "@rspack/binding";

import type { AssetConditions } from "../util/assetCondition";
import { create } from "./base";

export type SwcCssMinimizerRspackPluginOptions = {
	test?: AssetConditions;
	exclude?: AssetConditions;
	include?: AssetConditions;
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
