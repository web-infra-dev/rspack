const fs = require("fs");
import "./index.css";

it("should only import assets that included in chunks", () => {
	const list = fs.readdirSync(__dirname);
	const svgFiles = list.filter(item => item.endsWith("svg"));
	expect(svgFiles.length).toBe(1);
});
