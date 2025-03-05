const fs = require("fs");
const path = require("path");
const readCase = (name) => fs.readFileSync(path.resolve(__dirname, `${name}.mjs`), "utf-8");

it("reexport star from external module", function () {
	expect(readCase("case1")).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'case1.snap'));
	expect(readCase("case2")).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'case2.snap'));
	expect(readCase("case3")).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'case3.snap'));
	expect(readCase("case4")).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'case4.snap'));
});
