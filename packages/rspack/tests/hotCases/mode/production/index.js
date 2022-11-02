const fs = require("fs");

it("hotCases production build", () => {
	const files = fs.readdirSync(__dirname);
	expect(files.length).toBe(1);
});
