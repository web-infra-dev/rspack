const path = require("path");
const serializer = require("jest-serializer-path");
const normalizePaths = serializer.normalizePaths;
const rspackPath = path.resolve(__dirname, "../../../rspack");

export function replacePaths(input: string) {
	const rspackRoot = normalizePaths(rspackPath);
	return normalizePaths(input).split(rspackRoot).join("<RSPACK_ROOT>");
}
