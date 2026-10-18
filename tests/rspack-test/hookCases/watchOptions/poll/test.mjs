import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);
const coreDir = path.dirname(require.resolve("@rspack/core/package.json"));
const binding = require(require.resolve("@rspack/binding", { paths: [coreDir] }));

export default {
	description: "maps native watcher polling options to intervals",
	options() {
		return {
			experiments: { nativeWatcher: true },
			plugins: [compiler => {
				const Original = binding.NativeWatcher;
				const intervals = [];
				binding.NativeWatcher = class {
					constructor(options) {
						intervals.push(options.pollInterval);
					}
				};
				try {
					for (const poll of [true, false, undefined, 125, 0]) {
						const fs = new compiler.watchFileSystem.constructor(compiler.inputFileSystem);
						fs.getNativeWatcher({ poll });
					}
					expect(intervals).toEqual([5007, undefined, undefined, 125, 0]);
				} finally {
					binding.NativeWatcher = Original;
				}
			}]
		};
	}
};
