import {
	BuiltinPluginName,
	type RawLightningCssBrowsers,
	type RawLightningCssMinimizerRspackPluginOptions
} from "@rspack/binding";

import browserslist from "browserslist";
import {
	type Drafts,
	type FeatureOptions,
	type NonStandard,
	type PseudoClasses,
	type Targets,
	browserslistToTargets,
	toFeatures
} from "../builtin-loader/lightningcss";
import type { AssetConditions } from "../util/assetCondition";
import { create } from "./base";

export type LightningCssMinimizerRspackPluginOptions = {
	test?: AssetConditions;
	include?: AssetConditions;
	exclude?: AssetConditions;
	removeUnusedLocalIdents?: boolean;
	minimizerOptions?: {
		errorRecovery?: boolean;
		targets?: Targets | string[] | string;
		include?: FeatureOptions;
		exclude?: FeatureOptions;
		draft?: Drafts;
		nonStandard?: NonStandard;
		pseudoClasses?: PseudoClasses;
		unusedSymbols?: string[];
	};
};

export const LightningCssMinimizerRspackPlugin = create(
	BuiltinPluginName.LightningCssMinimizerRspackPlugin,
	(
		options?: LightningCssMinimizerRspackPluginOptions
	): RawLightningCssMinimizerRspackPluginOptions => {
		const { include, exclude, draft, nonStandard, pseudoClasses } =
			options?.minimizerOptions ?? {};
		const targets = options?.minimizerOptions?.targets ?? "fully supports es6"; // last not support es module chrome version
		return {
			test: options?.test,
			include: options?.include,
			exclude: options?.exclude,
			removeUnusedLocalIdents: options?.removeUnusedLocalIdents ?? true,
			minimizerOptions: {
				errorRecovery: options?.minimizerOptions?.errorRecovery ?? true,
				unusedSymbols: options?.minimizerOptions?.unusedSymbols ?? [],
				include: include ? toFeatures(include) : undefined,
				exclude: exclude
					? toFeatures(exclude)
					: // exclude all features, avoid downgrade css syntax when minimize
						// 1048575 = Features.Empty | Features.Nesting | ... | Features.LogicalProperties
						1048575,
				targets:
					typeof targets === "string" || Array.isArray(targets)
						? (browserslistToTargets(
								browserslist(targets)
							) as RawLightningCssBrowsers)
						: targets,
				draft: draft ? { customMedia: draft.customMedia ?? false } : undefined,
				nonStandard: nonStandard
					? {
							deepSelectorCombinator:
								nonStandard.deepSelectorCombinator ?? false
						}
					: undefined,
				pseudoClasses
			}
		};
	}
);
