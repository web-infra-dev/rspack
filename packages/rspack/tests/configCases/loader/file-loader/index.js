it("should compat with file-loader", () => {
	const fs = require("fs");
	const path = require("path");
	const dest = require("./logo.png");
	const source = fs.readFileSync(path.join(__dirname, "../logo.png"));
	const expected = fs.readFileSync(path.join(__dirname, dest));
	expect(source).toEqual(expected);
});
