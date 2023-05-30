import fs from "fs";
import path from "path";
import a from "./a.png?123";

it("should have a.png in dist", () => {
	expect(a).toBe("/a.png?123");
	const filePath = path.resolve(__dirname, "a.png");
	expect(fs.existsSync(filePath)).toBe(true);
});
