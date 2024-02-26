const path = require("path");
const fs = require("fs");

it("should compile successfully", () => {
	const exist = fs.existsSync(path.resolve(__dirname, "./temp"));
	expect(exist).toBeTruthy();
});
