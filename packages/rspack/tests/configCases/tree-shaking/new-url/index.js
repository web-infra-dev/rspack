import { a } from "./a";
const fs = require("fs");

it("should not shake the url import", () => {
	a();
	const file = fs.readFileSync(__filename).toString();
	// 3 = 1 time(for module id) + 2 time(referenced in another module + comment) + 1 time(in assertion)
	expect(countSubstringOccurrences(file, "a.wasm")).toBe(4);
});

function countSubstringOccurrences(string, substring) {
	if (substring.length === 0) {
		return 0;
	}

	let count = 0;
	let index = 0;

	while (index !== -1) {
		index = string.indexOf(substring, index);

		if (index !== -1) {
			count++;
			index += substring.length;
		}
	}

	return count;
}
