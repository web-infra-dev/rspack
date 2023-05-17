import { a } from "./lib";
import * as path from "path";
const fs = require("fs");

it("should not panic when trying to export * from a external module", () => {
	a;
	const content = fs.readFileSync(__filename);
	expect(content.includes("Buffer")).toBe(true);
});
