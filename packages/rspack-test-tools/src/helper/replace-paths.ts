import os from "node:os";
import path from "node:path";

const serializer = require("jest-serializer-path");
const normalizePaths = serializer.normalizePaths;
const rspackPath = path.resolve(__dirname, "../../../rspack");

export function replacePaths(input: string) {
	const paths = input.split("\\\\").join("\\");
	const rspackRoot: string = normalizePaths(rspackPath);

	if (os.platform() === "win32") {
		const winRspackRoot = rspackRoot.split("\\\\").join(path.win32.sep);
		return normalizePaths(paths)
			.split(rspackRoot)
			.join("<RSPACK_ROOT>")
			.split(winRspackRoot)
			.join("<RSPACK_ROOT>");
	}
	return normalizePaths(paths).split(rspackRoot).join("<RSPACK_ROOT>");
}
