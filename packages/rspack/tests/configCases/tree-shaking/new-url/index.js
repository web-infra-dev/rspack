import { a } from "./a";
import { b } from "./b";
const fs = require("fs");
const path = require("path");

it("should not shake the url import", () => {
	a();
	b();
	const file = fs.readFileSync(__filename).toString();
	// 4 = 1 time(for module id) + 2 time(referenced in another module + comment) + 1 time(in assertion)
	expect(countSubstringOccurrences(file, "a.wasm")).toBe(4);
	// 2 = 1 time(in comment) + 1 time(in assertion)
	expect(countSubstringOccurrences(file, "worker import")).toBe(2);
	expect(fs.existsSync(path.resolve(__dirname, "b_worker_js.js"))).toBe(true);
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
