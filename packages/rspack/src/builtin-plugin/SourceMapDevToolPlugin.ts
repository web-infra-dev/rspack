import {
	BuiltinPluginName,
	RawSourceMapDevToolPluginOptions
} from "@rspack/binding";

import { matchObject } from "../lib/ModuleFilenameHelpers";
import { create } from "./base";

/**
 * Include source maps for modules based on their extension (defaults to .js and .css).
 */
type Rules = Rule[] | Rule;
/**
 * Include source maps for modules based on their extension (defaults to .js and .css).
 */
type Rule = RegExp | string;

export interface SourceMapDevToolPluginOptions
	extends Omit<
		RawSourceMapDevToolPluginOptions,
		"test" | "include" | "exclude"
	> {
	exclude?: Rules;
	include?: Rules;
	test?: Rules;
}

export const SourceMapDevToolPlugin = create(
	BuiltinPluginName.SourceMapDevToolPlugin,
	(
		options: SourceMapDevToolPluginOptions
	): RawSourceMapDevToolPluginOptions => {
		const { test, include, exclude, ...rest } = options;

		const rawOptions: RawSourceMapDevToolPluginOptions = rest;

		if (test || include || exclude) {
			rawOptions.test = text => matchObject({ test, include, exclude }, text);
		}

		return rawOptions;
	},
	"compilation"
);
