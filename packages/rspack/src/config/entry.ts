import path from "path";

export type EntryRuntime = false | string;
export interface EntryDescription {
	import: EntryItem;
	runtime?: EntryRuntime;
}
export interface ResolvedEntryItem {
	import: string[];
	runtime?: string;
}
export type EntryItem = string[] | string;
export type EntryUnnamed = EntryItem;
export type Entry = EntryStatic;

export interface EntryObject {
	[k: string]: EntryItem | EntryDescription;
}
export type EntryStatic = EntryObject | EntryUnnamed;

export type ResolvedEntry = Record<string, ResolvedEntryItem>;

interface ResolveEntryContext {
	context: string;
}

export function resolveEntryOptions(
	options: Entry | undefined,
	context: ResolveEntryContext
): ResolvedEntry {
	if (typeof options === "undefined" || options === null) {
		return {
			main: normalizeEntryItem("main", {
				import: [path.resolve(context.context, "src", "index.js")]
			})
		};
	} else if (typeof options === "string") {
		return {
			main: normalizeEntryItem("main", {
				import: [options]
			})
		};
	} else if (Array.isArray(options)) {
		return {
			main: normalizeEntryItem("main", {
				import: [...options]
			})
		};
	} else if (typeof options === "object") {
		return Object.fromEntries(
			Object.entries({ ...options }).map(([key, value]) => {
				if (Array.isArray(value)) {
					return [
						key,
						normalizeEntryItem(key, {
							import: [...value]
						})
					];
				} else if (typeof value === "object") {
					return [key, normalizeEntryItem(key, value)];
				} else {
					return [
						key,
						normalizeEntryItem(key, {
							import: [value]
						})
					];
				}
			})
		);
	} else {
		return {};
	}
}

function normalizeEntryItem(
	entryName: string,
	entryItem: EntryDescription
): ResolvedEntryItem {
	return {
		import: Array.isArray(entryItem.import)
			? entryItem.import
			: [entryItem.import],
		runtime: entryItem.runtime === false ? undefined : entryItem.runtime
	};
}
