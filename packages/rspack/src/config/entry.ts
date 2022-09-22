import { Dev, getAdditionDevEntry } from "./dev";

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
