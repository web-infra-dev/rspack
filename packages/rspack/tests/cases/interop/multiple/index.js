import a from "./a";
import b from "./b";
const fs = require("fs");

it("should exports __esModule", function () {
	expect(exports.__esModule).toBe(true);
});

it("should have interop", function () {
	expect(a).not.toBeUndefined();
	expect(b).not.toBeUndefined();
});

it("should interop helper inject once", function () {
	const content = fs.readFileSync(__filename, "utf-8");
	const keyStr = content.match(/__webpack_require__\.ir/);
	expect(keyStr && keyStr.length).toBe(1);
});
