const path = require("path");

const CURRENT_CWD = process.cwd();
const ROOT = path.resolve(__dirname, "../../../../");

const quoteMeta = str => str.replace(/[-[\]\\/{}()*+?.^$|]/g, "\\$&");
const cwdRegExp = new RegExp(
	`${quoteMeta(CURRENT_CWD)}((?:\\\\)?(?:[a-zA-Z.\\-_]+\\\\)*)`,
	"g"
);
const escapedCwd = JSON.stringify(CURRENT_CWD).slice(1, -1);
const escapedCwdRegExp = new RegExp(
	`${quoteMeta(escapedCwd)}((?:\\\\\\\\)?(?:[a-zA-Z.\\-_]+\\\\\\\\)*)`,
	"g"
);
const normalize = str => {
	let normalizedStr = str;
	if (CURRENT_CWD.startsWith("/")) {
		normalizedStr = normalizedStr.replace(
			new RegExp(quoteMeta(CURRENT_CWD), "g"),
			"<cwd>"
		);
	} else {
		normalizedStr = normalizedStr.replace(
			cwdRegExp,
			(_, g) => `<cwd>${g.replace(/\\/g, "/")}`
		);
		normalizedStr = normalizedStr.replace(
			escapedCwdRegExp,
			(_, g) => `<cwd>${g.replace(/\\\\/g, "/")}`
		);
	}
	normalizedStr = normalizedStr.split(ROOT).join("<root>");
	normalizedStr = normalizedStr.replace(/:\d+:\d+/g, ":<line>:<row>");
	normalizedStr = normalizedStr.replace(
		/@@ -\d+,\d+ \+\d+,\d+ @@/g,
		"@@ ... @@"
	);
	return normalizedStr;
};

expect.addSnapshotSerializer({
	test(value) {
		return typeof value === "string";
	},
	print(received) {
		return normalize(received);
	}
});
