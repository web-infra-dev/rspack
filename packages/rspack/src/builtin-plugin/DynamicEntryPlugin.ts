import {
	BuiltinPluginName,
	type RawDynamicEntryPluginOptions
} from "@rspack/binding";

import type { EntryDynamicNormalized } from "../config";
import EntryOptionPlugin from "../lib/EntryOptionPlugin";
import { getRawEntryOptions } from "./EntryPlugin";
import { create } from "./base";

export const DynamicEntryPlugin = create(
	BuiltinPluginName.DynamicEntryPlugin,
	(
		context: string,
		entry: EntryDynamicNormalized
	): RawDynamicEntryPluginOptions => {
		return {
			context,
			entry: async () => {
				const result = await entry();
				return Object.entries(result).map(([name, desc]) => {
					const options = EntryOptionPlugin.entryDescriptionToOptions(
						{} as any,
						name,
						desc
					);
					return {
						import: desc.import!,
						options: getRawEntryOptions(options)
					};
				});
			}
		};
	},
	"make"
);
