import {
	BuiltinPluginName,
	RawEntryOptions,
	RawEntryPluginOptions
} from "@rspack/binding";
import { create } from "./base";
import {
	ChunkLoading,
	EntryRuntime,
	FilenameTemplate,
	LibraryOptions,
	PublicPath,
	getRawChunkLoading,
	getRawLibrary
} from "../config";
import { isNil } from "../util";

export type EntryOptions = {
	name?: string;
	runtime?: EntryRuntime;
	chunkLoading?: ChunkLoading;
	asyncChunks?: boolean;
	publicPath?: PublicPath;
	baseUri?: string;
	filename?: FilenameTemplate;
	library?: LibraryOptions;
	dependOn?: string[];
};
export const EntryPlugin = create(
	BuiltinPluginName.EntryPlugin,
	(
		context: string,
		entry: string,
		options: EntryOptions | string = ""
	): RawEntryPluginOptions => {
		let entryOptions =
			typeof options === "string" ? { name: options } : options;
		return {
			context,
			entry,
			options: getRawEntryOptions(entryOptions)
		};
	},
	"make"
);

export function getRawEntryOptions(entry: EntryOptions): RawEntryOptions {
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
		dependOn: entry.dependOn
	};
}
