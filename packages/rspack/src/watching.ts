import type { Compiler } from ".";
import type { FSWatcher } from "chokidar";
import { Stats } from ".";

class Watching {
	watcher: FSWatcher;
	compiler: Compiler;

	constructor(
		compiler: Compiler,
		watcher: FSWatcher,
		handler?: (error?: Error, stats?: Stats) => void
	) {
		this.watcher = watcher;
		this.compiler = compiler;

		const build = () => {
			const begin = Date.now();
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
					this.watch([
						...this.compiler.compilation.fileDependencies,
						...this.compiler.compilation.contextDependencies,
						...this.compiler.compilation.missingDependencies
					]);
					console.log("build success, time cost", Date.now() - begin, "ms");
				});
			});
		};

		watcher.on("ready", build);

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
					this.watch([
						...this.compiler.compilation.fileDependencies,
						...this.compiler.compilation.contextDependencies,
						...this.compiler.compilation.missingDependencies
					]);

					console.log("rebuild success, time cost", Date.now() - begin, "ms");
				});
			};

			if (compiler.options.devServer) {
				rebuildWithFilepaths([...pendingChangedFilepaths, changedFilepath]);
			} else {
				build();
			}
		});
	}

	watch(paths: string[]) {
		this.watcher.add(paths);
	}

	close(callback?: () => void) {
		this.compiler.watching = undefined;
		this.watcher.close().then(callback);
	}

	invalidate() {}
}

export default Watching;
