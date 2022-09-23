import { Dev, getAdditionDevEntry } from "./dev";
import { mapValues } from "../utils";
import path from "path";

export type Entry = Record<string, string>;

export type ResolvedEntry = Record<string, string>;

interface ResolveEntryContext {
	dev: Dev;
}

export function resolveEntry(
	entry = {},
	context: ResolveEntryContext
): ResolvedEntry {
	if (!context.dev) {
		return entry;
	} else {
		return {
			...entry,
			...getAdditionDevEntry()
		};
	}
}
export function resolveEntryOptions(
	entry: Record<string, string> | string,
	options: { context: string; dev: boolean }
) {
	if (typeof entry === "string") {
		entry = {
			main: entry
		};
	}
	return {
		...mapValues(entry, item => path.resolve(options.context, item)),
		...(options.dev ? getAdditionDevEntry() : {})
	};
}
