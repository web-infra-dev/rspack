import fs from "node:fs";
import path from "node:path";

const mtime = new Date("2020-01-01T00:00:00Z");
let compilerIndex = 0;

export default {
	mode: "development",
	incremental: false,
	cache: {
		type: "persistent",
		storage: {
			type: "filesystem",
			directory: path.join(import.meta.dirname, ".cache")
		}
	},
	experiments: {
		newCache: false,
		useInputFileSystem: [/[\\/]file\.js$/]
	},
	plugins: [
		compiler => {
			const source = path.join(compiler.context, "file.js");
			const ctimeMs = mtime.getTime() + (compilerIndex++ < 2 ? 1000 : 2000);
			fs.utimesSync(source, mtime, mtime);
			compiler.inputFileSystem = {
				...fs,
				stat(filename, callback) {
					fs.stat(filename, (error, stats) => {
						// Use fixed change times so the test does not depend on clock precision.
						if (stats && filename === source) stats.ctimeMs = ctimeMs;
						callback(error, stats);
					});
				}
			};
		}
	]
};
