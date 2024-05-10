const path = require("path");

it("should load entry with custom filename", async () => {
	expect(path.basename(__filename)).toBe("my-main.js");
});
