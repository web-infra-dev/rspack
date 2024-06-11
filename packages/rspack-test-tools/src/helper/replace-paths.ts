const path = require("path");
const serializer = require("jest-serializer-path");
const normalizePaths = serializer.normalizePaths;
const rspackPath = path.resolve(__dirname, "../../../rspack");
const os = require("os");

export function replacePaths(input: string) {
	const rspackRoot = normalizePaths(rspackPath);
	if (os.platform() === "win32") {
		input = input.split("\\\\").join(path.win32.sep);
	}
	return normalizePaths(input).split(rspackRoot).join("<RSPACK_ROOT>");
}
