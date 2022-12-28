import type { WatchOptions } from "chokidar";

export type Watch = WatchOptions;

export type ResolvedWatch = WatchOptions;

export function resolveWatchOption(watch: Watch = {}): ResolvedWatch {
	const ignored = watch.ignored ?? [
		"**/dist/**",
		"**/node_modules/**",
		"**/.git/**"
	];
	return {
		...watch,
		ignored
	};
}
