const fs = require("fs");

it("should load async.js with require", async () => {
	const chunk = await import("./async");
	expect(chunk.default).toBe(42);
	const code = await fs.promises.readFile(__filename, "utf-8");
	expect(code.includes("__webpack_require__.f.require")).toBe(true);
});
