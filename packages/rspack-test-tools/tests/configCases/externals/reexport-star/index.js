const fs = require("fs");
const path = require("path");
const readCase = (name)=> fs.readFileSync(path.resolve(__dirname, `${name}.mjs`), "utf-8");

it("reexport star from external module", function () {
	expect(readCase("case1")).toMatchSnapshot();
	expect(readCase("case2")).toMatchSnapshot();
	expect(readCase("case3")).toMatchSnapshot();
	expect(readCase("case4")).toMatchSnapshot();
});
