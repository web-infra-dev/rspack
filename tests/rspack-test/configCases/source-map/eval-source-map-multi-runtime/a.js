import { a } from "./lib";

it("basic", () => {
	expect(a).toBe("a");
	const fs = require("fs");
	const source = fs.readFileSync(__filename, "utf-8");
	const regex = /sourceMappingURL\s*=\s*data:application\/json;charset=utf-8;base64,(.*)\\n\/\/#/g;
	const files = [];
	let match;
	while (match = regex.exec(source)) {
		const base64 = match[1];
		const map = JSON.parse(Buffer.from(base64, "base64").toString("utf-8"));
		files.push(map.file);
	}
	// Align with webpack: `file` is the module id (`${id}.js` for numeric ids),
	// not the absolute path
	expect(files).toHaveLength(2);
	for (const file of files) {
		expect(file).toMatch(/^\d+\.js$/);
	}
});
