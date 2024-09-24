import fs from "fs";
import path from "path";

it("static import from external module should be tree shaking friendly", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "consume.mjs"), "utf-8");
	expect(content).not.toContain("22222");
});
