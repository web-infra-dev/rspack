import path from "node:path";
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
	expect(files).toContain(path.join(CONTEXT, "a.js"));
	expect(files).toContain(path.join(CONTEXT, "lib.js"));
});
