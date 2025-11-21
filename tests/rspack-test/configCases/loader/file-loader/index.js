it("should compat with file-loader", () => {
	const fs = require("fs");
	const source = fs.readFileSync(
		__dirname + "/" + require("./logo.png"),
		"utf-8"
	);
	expect(source).toEqual("");
});
