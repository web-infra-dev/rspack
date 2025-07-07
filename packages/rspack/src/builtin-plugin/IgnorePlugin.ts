import {
	BuiltinPluginName,
	type RawIgnorePluginOptions
} from "@rspack/binding";
import { anyFunction } from "../config/utils";
import { z } from "../config/zod";
import { memoize } from "../util/memoize";
import { validate } from "../util/validate";
import { create } from "./base";

export type IgnorePluginOptions =
	| {
			/** A RegExp to test the resource against. */
			resourceRegExp: NonNullable<RawIgnorePluginOptions["resourceRegExp"]>;

			/** A RegExp to test the context (directory) against. */
			contextRegExp?: RawIgnorePluginOptions["contextRegExp"];
	  }
	| {
			/** A Filter function that receives `resource` and `context` as arguments, must return boolean. */
			checkResource: NonNullable<RawIgnorePluginOptions["checkResource"]>;
	  };

const getIgnorePluginOptionsSchema = memoize(
	() =>
		z.union([
			z.object({
				contextRegExp: z.instanceof(RegExp).optional(),
				resourceRegExp: z.instanceof(RegExp)
			}),
			z.object({
				checkResource: anyFunction
			})
		]) satisfies z.ZodType<IgnorePluginOptions>
);

export const IgnorePlugin = create(
	BuiltinPluginName.IgnorePlugin,
	(options: IgnorePluginOptions): RawIgnorePluginOptions => {
		validate(options, getIgnorePluginOptionsSchema);

		return options;
	}
);
