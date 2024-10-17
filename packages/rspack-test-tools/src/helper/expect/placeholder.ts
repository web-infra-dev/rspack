import path from "node:path";
import { createSnapshotSerializer } from "path-serializer";

const placeholderSerializer = createSnapshotSerializer({
	root: path.resolve(__dirname, "../../../../../"),
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
