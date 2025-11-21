import fs from "fs";
import path from "path";

it("should have single runtime chunk", async () => {
	await import("./bootstrap");
	const code = await fs.promises.readFile(path.resolve(__dirname, "container-a.js"), "utf-8");
	expect(code).not.toContain("=>");
	expect(code).not.toContain("const");
	expect(code).not.toContain("let");
});
