it("should load a raw buffer if `loader.raw` is sat to `true`", () => {
	const fs = require("fs");
	const path = require("path");

	// FIXME: We should align this with target `Node`, currently the `__webpack_require__.p` is not defined for the `Node`. cc @underfin
	const dest = require("./logo.png").replace("undefined", "");

	const source = fs.readFileSync(path.join(__dirname, "../logo.png"));
	const expected = fs.readFileSync(path.join(__dirname, dest));
	expect(source).toEqual(expected);
});
