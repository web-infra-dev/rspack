import chokidar from "chokidar";
import type { WatchOptions, FSWatcher } from "chokidar";
import type { ResolvedRspackOptions } from "../config";

export function createWatcher(options: ResolvedRspackOptions): FSWatcher {
	const watchOptions: WatchOptions = {
		ignored: ["**/node_modules/**", "**/.git/**", "**/dist/**", "**/lib/**"]
	};

	const watcher = chokidar.watch(options.context, watchOptions);
	return watcher;
}
