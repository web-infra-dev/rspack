import { getAdditionDevEntry } from "./devServer";
import path from "path";

export type Entry = string | string[] | Record<string, string | string[]>;
export type ResolvedEntry = Record<string, string[]>;

interface ResolveEntryContext {
	context: string;
	dev: boolean; // TODO: dev: DEV
}

export function resolveEntryOptions(
	options: Entry,
	context: ResolveEntryContext
): ResolvedEntry {
	const additionDevEntry = context.dev ? getAdditionDevEntry() : {};
	if (typeof options === "undefined" || options === null) {
		return {
			main: [path.resolve(context.context, "src", "index.js")],
			...additionDevEntry
		};
	} else if (typeof options === "string") {
		return {
			main: [options],
			...additionDevEntry
		};
	} else if (Array.isArray(options)) {
		return {
			main: options,
			...additionDevEntry
		};
	} else if (typeof options === "object") {
		return Object.fromEntries(
			Object.entries({ ...options, ...additionDevEntry }).map(
				([key, value]) => {
					if (Array.isArray(value)) {
						return [key, value];
					} else {
						return [key, [value]];
					}
				}
			)
		);
	} else {
		return {};
	}
}

// export function resolveEntryOptions(
// 	entry: Record<string, string> | string,
// 	options: { context: string; dev: boolean }
// ) {
// 	if (typeof entry === "string") {
// 		entry = {
// 			main: entry
// 		};
// 	}
// 	return {
// 		...mapValues(entry, item => path.resolve(options.context, item)),
// 		...(options.dev ? getAdditionDevEntry() : {})
// 	};
// }
