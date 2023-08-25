import {
	RawBannerCondition,
	RawBannerConditions,
	RawBannerConfig
} from "@rspack/binding";
import { BuiltinPluginKind, create } from "./base";

type BannerCondition = string | RegExp;
type BannerConditions = BannerCondition | BannerCondition[];
export type BannerPluginOptions =
	| string
	| {
			banner: string;
			entryOnly?: boolean;
			footer?: boolean;
			raw?: boolean;
			test?: BannerConditions;
			exclude?: BannerConditions;
			include?: BannerConditions;
	  };

function getBannerCondition(condition: BannerCondition): RawBannerCondition {
	if (typeof condition === "string") {
		return {
			type: "string",
			stringMatcher: condition
		};
	}
	if (condition instanceof RegExp) {
		return {
			type: "regexp",
			regexpMatcher: condition.source
		};
	}
	throw new Error("unreachable: condition should be one of string, RegExp");
}

function getBannerConditions(
	condition?: BannerConditions
): RawBannerConditions | undefined {
	if (!condition) return undefined;

	if (Array.isArray(condition)) {
		return {
			type: "array",
			arrayMatcher: condition.map(i => getBannerCondition(i))
		};
	}

	return getBannerCondition(condition);
}

export const BannerPlugin = create(
	BuiltinPluginKind.Banner,
	(bannerConfig: BannerPluginOptions): RawBannerConfig => {
		if (typeof bannerConfig === "string") {
			return {
				banner: bannerConfig
			};
		}

		return {
			...bannerConfig,
			test: getBannerConditions(bannerConfig.test),
			include: getBannerConditions(bannerConfig.include),
			exclude: getBannerConditions(bannerConfig.exclude)
		};
	}
);
