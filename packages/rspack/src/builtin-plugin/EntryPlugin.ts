import { RawEntryOptions, RawEntryPluginOptions } from "@rspack/binding";
import { BuiltinPluginKind, create } from "./base";
import { ChunkLoading, EntryRuntime, Filename, PublicPath } from "..";

export type EntryOptions = {
	name?: string;
	runtime?: EntryRuntime;
	chunkLoading?: ChunkLoading;
	asyncChunks?: boolean;
	publicPath?: PublicPath;
	baseUri?: string;
	filename?: Filename;
};
export const EntryPlugin = create(
	BuiltinPluginKind.Entry,
	(
		context: string,
		entry: string,
		options: EntryOptions
	): RawEntryPluginOptions => {
		return {
			context,
			entry,
			options: getRawEntryOptions(options)
		};
	}
);

function getRawEntryOptions(entry: EntryOptions): RawEntryOptions {
	const runtime = entry.runtime;
	const chunkLoading = entry.chunkLoading;
	return {
		name: entry.name,
		publicPath: entry.publicPath,
		baseUri: entry.baseUri,
		runtime: runtime === false ? undefined : runtime,
		chunkLoading: chunkLoading === false ? "false" : chunkLoading,
		asyncChunks: entry.asyncChunks,
		filename: entry.filename
	};
}
