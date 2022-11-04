import path from "path";

export type Entry = string | string[] | Record<string, string | string[]>;
export type ResolvedEntry = Record<string, string[]>;

interface ResolveEntryContext {
	context: string;
}

export function resolveEntryOptions(
	options: Entry,
	context: ResolveEntryContext
): ResolvedEntry {
	if (typeof options === "undefined" || options === null) {
		return {
			main: [path.resolve(context.context, "src", "index.js")]
		};
	} else if (typeof options === "string") {
		return {
			main: [options]
		};
	} else if (Array.isArray(options)) {
		return {
			main: [...options]
		};
	} else if (typeof options === "object") {
		return Object.fromEntries(
			Object.entries({ ...options }).map(([key, value]) => {
				if (Array.isArray(value)) {
					return [key, [...value]];
				} else {
					return [key, [value]];
				}
			})
		);
	} else {
		return {};
	}
}
