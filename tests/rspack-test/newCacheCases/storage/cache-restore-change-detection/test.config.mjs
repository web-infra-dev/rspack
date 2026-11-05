import { execFile } from "node:child_process";
import path from "node:path";
import { promisify } from "node:util";

const run = promisify(execFile);

export default {
	timeout: 60_000,
	moduleScope(scope, stats, options) {
		scope.watchAfterCacheRestore = async () => {
			// Neither process can inherit native paths from the other's path interner.
			for (const phase of ["warm", "watch"]) {
				await run(process.execPath, [
					path.join(import.meta.dirname, "watch.mjs"),
					phase,
					path.join(options.output.path, "native-watch")
				], {
					cwd: options.context,
					windowsHide: true,
					timeout: 30_000
				});
			}
		};
		return scope;
	}
};
