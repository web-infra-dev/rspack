const path = require("node:path");
const fs = require("node:fs");
const { parse, Lang } = require("@ast-grep/napi");

const dist = fs.readFileSync(
	require.resolve(path.resolve(__dirname, "../../dist/index.js")),
	"utf-8"
);
const root = parse(Lang.JavaScript, dist).root();
const edits = [...require("./binding").replaceBinding(root)];

fs.writeFileSync(
	require.resolve(path.resolve(__dirname, "../../dist/index.js")),
	root.commitEdits(edits)
);
