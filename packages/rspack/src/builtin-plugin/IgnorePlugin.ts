import {
	BuiltinPluginName,
	type RawIgnorePluginOptions
} from "@rspack/binding";
import { getIgnorePluginOptionsSchema } from "../schema/plugins";
import { validate } from "../schema/validate";
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

export const IgnorePlugin = create(
	BuiltinPluginName.IgnorePlugin,
	(options: IgnorePluginOptions): RawIgnorePluginOptions => {
		validate(options, getIgnorePluginOptionsSchema);

		return options;
	}
);
