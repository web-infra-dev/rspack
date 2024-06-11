import os from "os";
import path from "path";

const serializer = require("jest-serializer-path");
const normalizePaths = serializer.normalizePaths;
const rspackPath = path.resolve(__dirname, "../../../rspack");

export function replacePaths(input: string) {
	const rspackRoot: string = normalizePaths(rspackPath);
	if (os.platform() === "win32") {
		const winRspackRoot = rspackRoot.split(path.posix.sep).join(path.win32.sep);
		console.log(input);
		console.log(winRspackRoot);
		return normalizePaths(input)
			.split(rspackRoot)
			.join("<RSPACK_ROOT>")
			.split(winRspackRoot)
			.join("<RSPACK_ROOT>");
	}
	return normalizePaths(input).split(rspackRoot).join("<RSPACK_ROOT>");
}
