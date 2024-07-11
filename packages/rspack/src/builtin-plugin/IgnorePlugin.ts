import {
	BuiltinPluginName,
	type RawIgnorePluginOptions
} from "@rspack/binding";
import { z } from "zod";

import { validate } from "../util/validate";
import { create } from "./base";

export type IgnorePluginOptions =
	| {
			resourceRegExp: NonNullable<RawIgnorePluginOptions["resourceRegExp"]>;
			contextRegExp?: RawIgnorePluginOptions["contextRegExp"];
	  }
	| {
			checkResource: NonNullable<RawIgnorePluginOptions["checkResource"]>;
	  };

const IgnorePluginOptions = z.union([
	z.object({
		contextRegExp: z.instanceof(RegExp).optional(),
		resourceRegExp: z.instanceof(RegExp)
	}),
	z.object({
		checkResource: z.function(z.tuple([z.string(), z.string()]), z.boolean())
	})
]);

export const IgnorePlugin = create(
	BuiltinPluginName.IgnorePlugin,
	(options: IgnorePluginOptions): RawIgnorePluginOptions => {
		validate(options, IgnorePluginOptions);

		return options;
	}
);
