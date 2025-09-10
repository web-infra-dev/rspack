import fs from "fs";

import("external");

it("external module should at main chunk", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content.includes('define(["external"]')).toBe(true);
});
