import fs from "fs";
import path from "path";

import("external");

it("external module should at main chunk", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "main.js"), "utf-8");
	expect(content.startsWith('define(["external"]')).toBe(true);
});
