import {
	RawSwcJsMinimizerRspackPluginOptions,
	RawSwcJsMinimizerRule,
	RawSwcJsMinimizerRules
} from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";

type MinifyCondition = string | RegExp;
type MinifyConditions = MinifyCondition | MinifyCondition[];
export type SwcJsMinimizerRspackPluginOptions = {
	passes?: number;
	dropConsole?: boolean;
	keepClassNames?: boolean;
	keepFnNames?: boolean;
	pureFuncs?: Array<string>;
	extractComments?: boolean | RegExp;
	comments?: false | "all" | "some";
	asciiOnly?: boolean;
	test?: MinifyConditions;
	exclude?: MinifyConditions;
	include?: MinifyConditions;
};

function getRawSwcJsMinimizerRule(
	condition: MinifyCondition
): RawSwcJsMinimizerRule {
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

function getRawSwcJsMinimizerRules(
	condition?: MinifyConditions
): RawSwcJsMinimizerRules | undefined {
	if (!condition) return undefined;

	if (Array.isArray(condition)) {
		return {
			type: "array",
			arrayMatcher: condition.map(i => getRawSwcJsMinimizerRule(i))
		};
	}

	return getRawSwcJsMinimizerRule(condition);
}

export const SwcJsMinimizerRspackPlugin = create(
	BuiltinPluginName.SwcJsMinimizerRspackPlugin,
	(
		options?: SwcJsMinimizerRspackPluginOptions
	): RawSwcJsMinimizerRspackPluginOptions => {
		return {
			passes: options?.passes ?? 1,
			dropConsole: options?.dropConsole ?? false,
			keepClassNames: options?.keepClassNames ?? false,
			keepFnNames: options?.keepFnNames ?? false,
			pureFuncs: options?.pureFuncs ?? [],
			comments: options?.comments ? options.comments : "false",
			asciiOnly: options?.asciiOnly ?? false,
			extractComments: options?.extractComments
				? String(options.extractComments)
				: undefined,
			test: getRawSwcJsMinimizerRules(options?.test),
			include: getRawSwcJsMinimizerRules(options?.include),
			exclude: getRawSwcJsMinimizerRules(options?.exclude)
		};
	}
);
