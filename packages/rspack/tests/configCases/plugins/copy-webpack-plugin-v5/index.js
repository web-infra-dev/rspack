const fs = require("fs");
const path = require("path");

it("copy-webpack-plugin-v5", () => {
	const copied = fs.readFileSync(path.resolve(__dirname, "stuff.txt"), "utf-8");
	expect(copied).toBe("from\nto");
});
