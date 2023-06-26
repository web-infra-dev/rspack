import { c } from "./package";
import * as path from "path";
const fs = require("fs");

it("should only import assets that included in chunks", () => {
	c;
	const list = fs.readdirSync(__dirname);
	const svgFiles = list.filter(item => item.endsWith("svg"));
	expect(svgFiles.length).toBe(0);
});
