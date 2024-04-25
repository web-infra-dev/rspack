import "./a.mjs";

it("should retain require callee", () => {
	const fs = require("fs");
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content).toContain(`require('./in-exists')`);
});
