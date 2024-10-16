import path from "node:path";
const { createSnapshotSerializer } = require("path-serializer");

const placeholderSerializer = createSnapshotSerializer({
	workspace: path.resolve(__dirname, "../../../../"),
	replace: [
		{
			match: path.resolve(__dirname, "../../../rspack"),
			mark: "rspack"
		},
		{
			match: path.resolve(__dirname, "../../"),
			mark: "test_tools"
		},
		{
			match: /:\d+:\d+-\d+:\d+/g,
			mark: "line_col_range"
		},
		{
			match: /:\d+:\d+/g,
			mark: "line_col"
		}
	]
});

export const normalizePlaceholder = placeholderSerializer.print;
