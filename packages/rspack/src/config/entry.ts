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
	const additionDevEntry = context.dev ? getAdditionDevEntry() : [];
	if (typeof options === "undefined" || options === null) {
		return {
			main: [
				...additionDevEntry,
				path.resolve(context.context, "src", "index.js")
			]
		};
	} else if (typeof options === "string") {
		return {
			main: [...additionDevEntry, options]
		};
	} else if (Array.isArray(options)) {
		return {
			main: [...additionDevEntry, ...options]
		};
	} else if (typeof options === "object") {
		return Object.fromEntries(
			Object.entries({ ...options }).map(([key, value]) => {
				if (Array.isArray(value)) {
					return [key, [...additionDevEntry, ...value]];
				} else {
					return [key, [...additionDevEntry, value]];
				}
			})
		);
	} else {
		return {};
	}
}
