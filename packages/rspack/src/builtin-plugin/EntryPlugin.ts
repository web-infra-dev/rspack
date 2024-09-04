import {
	BuiltinPluginName,
	type JsEntryOptions,
	type JsEntryPluginOptions
} from "@rspack/binding";

import {
	type ChunkLoading,
	type EntryRuntime,
	type Filename,
	type Layer,
	type LibraryOptions,
	type PublicPath,
	getRawChunkLoading,
	getRawLibrary
} from "../config";
import { isNil } from "../util";
import { create } from "./base";

export type EntryOptions = {
	name?: string;
	runtime?: EntryRuntime;
	chunkLoading?: ChunkLoading;
	asyncChunks?: boolean;
	publicPath?: PublicPath;
	baseUri?: string;
	filename?: Filename;
	library?: LibraryOptions;
	layer?: Layer;
	dependOn?: string[];
};
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
