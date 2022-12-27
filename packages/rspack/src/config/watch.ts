export type Watch = {
	ignored?: string[];
};

export type ResolvedWatch = {
	ignored: string[];
};

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
