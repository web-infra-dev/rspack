import {
	BuiltinPluginName,
	type JsEntryOptions,
	type JsEntryPluginOptions
} from "@rspack/binding";

import {
	type EntryDescriptionNormalized,
	getRawChunkLoading,
	getRawLibrary
} from "../config";
import { isNil } from "../util";
import { create } from "./base";

/**
 * Options for the `EntryPlugin`.
 */
export type EntryOptions = Omit<EntryDescriptionNormalized, "import"> & {
	/**
	 * The name of the entry chunk.
	 */
	name?: string;
};

/**
 * The entry plugin that will handle creation of the `EntryDependency`.
 * It adds an entry chunk on compilation. The chunk is named `options.name` and
 * contains only one module (plus dependencies). The module is resolved from
 * `entry` in `context` (absolute path).
 */
export const EntryPlugin = create(
	BuiltinPluginName.EntryPlugin,
	(
		context: string,
		entry: string,
		options: EntryOptions | string = ""
	): JsEntryPluginOptions => {
		const entryOptions =
			typeof options === "string" ? { name: options } : options;
		return {
			context,
			entry,
			options: getRawEntryOptions(entryOptions)
		};
	},
	"make"
);

export function getRawEntryOptions(entry: EntryOptions): JsEntryOptions {
	const runtime = entry.runtime;
	const chunkLoading = entry.chunkLoading;
	return {
		name: entry.name,
		publicPath: entry.publicPath,
		baseUri: entry.baseUri,
		runtime,
		chunkLoading: !isNil(chunkLoading)
			? getRawChunkLoading(chunkLoading)
			: undefined,
		asyncChunks: entry.asyncChunks,
		filename: entry.filename,
		library: entry.library && getRawLibrary(entry.library),
		layer: entry.layer ?? undefined,
		dependOn: entry.dependOn
	};
}
