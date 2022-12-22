import type { Compiler } from ".";
import type { FSWatcher } from "chokidar";
import { Stats } from ".";

class Watching {
	wather: FSWatcher;
	constructor(
		compiler: Compiler,
		watcher: FSWatcher,
		handler?: (error?: Error, stats?: Stats) => void
	) {
		this.wather = watcher;

		const begin = Date.now();

		watcher.on("ready", () => {
			compiler.build((error, rawStats) => {
				if (error && handler) {
					return handler(error);
				} else if (error) {
					throw error;
				}
				const stats = new Stats(rawStats, compiler.compilation);
				compiler.hooks.done.callAsync(stats, () => {
					if (handler) {
						handler(undefined, stats);
					}
					console.log("build success, time cost", Date.now() - begin, "ms");
				});
			});
		});

		let pendingChangedFilepaths = new Set<string>();
		let isBuildFinished = true;

		// TODO: should use aggregated
		watcher.on("change", async changedFilepath => {
			// TODO: only build because we lack the snapshot info of file.
			// TODO: it means there a lot of things to do....

			// store the changed file path, it may or may not be consumed right now
			if (!isBuildFinished) {
				pendingChangedFilepaths.add(changedFilepath);
				console.log(
					"hit change but rebuild is not finished, caching files: ",
					pendingChangedFilepaths
				);
				return;
			}

			const rebuildWithFilepaths = (changedFilepath: string[]) => {
				// Rebuild finished, we can start to rebuild again
				isBuildFinished = false;
				console.log("hit change and start to build:", changedFilepath);

				const begin = Date.now();
				compiler.rebuild(changedFilepath, (error, rawStats) => {
					isBuildFinished = true;

					const hasPending = Boolean(pendingChangedFilepaths.size);

					// If we have any pending task left, we should rebuild again with the pending files
					if (hasPending) {
						const pending = [...pendingChangedFilepaths];
						pendingChangedFilepaths.clear();
						rebuildWithFilepaths(pending);
					}
					if (error && handler) {
						return handler(error);
					} else if (error) {
						throw error;
					}
					const stats = new Stats(rawStats, compiler.compilation);
					if (handler) {
						handler(undefined, stats);
					}

					console.log("rebuild success, time cost", Date.now() - begin, "ms");
				});
			};

			rebuildWithFilepaths([...pendingChangedFilepaths, changedFilepath]);
		});
	}

	close(callback?: () => void) {
		this.wather.close().then(callback);
	}

	invalidate() {}
}

export default Watching;
