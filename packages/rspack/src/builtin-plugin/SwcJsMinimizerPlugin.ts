import {
	RawMinification,
	RawMinificationCondition,
	RawMinificationConditions
} from "@rspack/binding";
import { BuiltinPluginKind, create } from "./base";

type MinifyCondition = string | RegExp;
type MinifyConditions = MinifyCondition | MinifyCondition[];
export type SwcJsMinimizerPluginOptions = {
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

function getMinifyCondition(
	condition: MinifyCondition
): RawMinificationCondition {
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

function getMinifyConditions(
	condition?: MinifyConditions
): RawMinificationConditions | undefined {
	if (!condition) return undefined;

	if (Array.isArray(condition)) {
		return {
			type: "array",
			arrayMatcher: condition.map(i => getMinifyCondition(i))
		};
	}

	return getMinifyCondition(condition);
}

export const SwcJsMinimizerPlugin = create(
	BuiltinPluginKind.SwcJsMinimizer,
	(options?: SwcJsMinimizerPluginOptions): RawMinification => {
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
			test: getMinifyConditions(options?.test),
			include: getMinifyConditions(options?.include),
			exclude: getMinifyConditions(options?.exclude)
		};
	}
);
