const fs = require("fs");
const path = require("path");

it("should have basic create module data", () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, "./createData.json"),
		"utf-8"
	);
	const createData = JSON.parse(content);
	expect(createData).toBeDefined();
	expect(typeof createData.matchResource).toBe("string");
	expect(typeof createData.settings).toBe("object");
});
