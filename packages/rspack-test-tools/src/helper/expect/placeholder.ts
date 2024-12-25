import path from "node:path";
import { createSnapshotSerializer } from "path-serializer";

// 1. escapeEOL \r\n -> \n
// 2. replace <RSPACK_ROOT> etc
// 3. transform win32 sep
const placeholderSerializer = createSnapshotSerializer({
	root: __dirname.includes("node_modules")
		? // Use `process.cwd()` when using outside Rspack
			process.cwd()
		: path.resolve(__dirname, "../../../../../"),
	replace: [
		{
			match: path.resolve(__dirname, "../../../"),
			mark: "test_tools_root"
		},
		{
			match: path.resolve(__dirname, "../../../../rspack"),
			mark: "rspack_root"
		},
		{
			match: /:\d+:\d+-\d+:\d+/g,
			mark: "line_col_range"
		},
		{
			match: /:\d+:\d+/g,
			mark: "line_col"
		}
	],
	features: {
		replaceWorkspace: false,
		addDoubleQuotes: false,
		escapeDoubleQuotes: false
	}
});

export const normalizePlaceholder = placeholderSerializer.serialize;
