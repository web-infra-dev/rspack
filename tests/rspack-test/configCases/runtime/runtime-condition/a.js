import { val, val2b } from "./shared";

it("should have correct runtime id", () => {
	expect(val).toBe(84);
	expect(val2b).toBe(42);
	expect(__webpack_require__.j).toBe("a-runtime");
});

it("should include runtime condition check code", () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");

	const source = fs.readFileSync(
		path.join(
			__dirname,
			"./shared.js"
		),
		"utf-8"
	);
	expect(source).toContain(`"a-runtime" == __webpack_require__.j`)
	expect(source).toContain(`"b-runtime" == __webpack_require__.j`);
	expect(source).toContain(`/^[ab]x\\-name$/.test(__webpack_require__.j)`);
})
