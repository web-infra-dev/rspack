import { a } from "./a";
import { b } from "./b";
const fs = require("fs");
const path = require("path");

it("should not shake the url import", () => {
	a();
	b();
	const file = fs.readFileSync(__filename).toString();
	expect(fs.readdirSync(__dirname).some(file => file.endsWith(".wasm"))).toBe(true);
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
