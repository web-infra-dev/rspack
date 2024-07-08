import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export type ProvidePluginOptions = Record<string, string | string[]>;
export const ProvidePlugin = create(
	BuiltinPluginName.ProvidePlugin,
	(provide: ProvidePluginOptions): Record<string, string[]> => {
		const entries = Object.entries(provide).map(([key, value]) => {
			if (typeof value === "string") {
				value = [value];
			}
			return [key, value];
		});
		return Object.fromEntries(entries);
	}
);
