require("./index.css");

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

function test() {
	return 123;
}

it("basic", () => {
	expect(test()).toBe(123);
});

it("format", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content).not.toMatch("\n");
});

it("css", () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, "bundle0.css"),
		"utf-8"
	);
	expect(content).not.toMatch("\n");
});
