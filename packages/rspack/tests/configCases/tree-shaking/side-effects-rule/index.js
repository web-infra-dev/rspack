import { c } from "./package";
const fs = require("fs");

it("should override sideEffects in package.json", () => {
	c;
	const file = fs.readFileSync(__filename);

	expect(countSubstring(file, "b.js")).toBe(1);
	expect(countSubstring(file, "a.js")).toBe(1);
});

function countSubstring(str, subStr) {
	let count = 0;
	let index = str.indexOf(subStr);

	while (index !== -1) {
		count++;
		index = str.indexOf(subStr, index + 1);
	}

	return count;
}
