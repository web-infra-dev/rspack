it("should load a raw buffer if `loader.raw` is sat to `true`", () => {
	const fs = require("fs");
	const source = fs.readFileSync(
		__dirname + "/" + require("./logo.png"),
		"utf-8"
	);
	expect(source).toEqual("");
});
