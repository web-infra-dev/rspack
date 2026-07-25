import { val, val2b } from "./shared";
import {
	checkedA,
	checkedB,
	checkedC,
	checkedD,
	checkedE,
	checkedF,
	checkedG,
	checkedH,
	setCheckedA,
	checkedMissing
} from "./checked-shared";
import { smallCheckedA, smallCheckedB } from "./small-checked-shared";

const getGeneratedModule = (source, request) => {
	const start = source.indexOf(`"${request}"`);
	expect(start).toBeGreaterThanOrEqual(0);
	const remaining = source.slice(start);
	const end = remaining.indexOf('\n},\n"');
	return end < 0 ? remaining : remaining.slice(0, end);
};

it("should have correct runtime id", () => {
	expect(val).toBe(84);
	expect(val2b).toBe(42);
	expect(__webpack_require__.j).toBe("a-runtime");
});

it("should preserve checked reexport semantics", () => {
	expect([
		checkedA,
		checkedB,
		checkedC,
		checkedD,
		checkedE,
		checkedF,
		checkedG,
		checkedH
	]).toEqual([1, 2, 3, 4, 5, 6, 7, 8]);
	expect(checkedMissing).toBeUndefined();
	expect([smallCheckedA, smallCheckedB]).toEqual([1, 2]);
	setCheckedA(101);
	expect(checkedA).toBe(101);
	setCheckedA(1);
	expect(checkedA).toBe(1);
});

it("should include runtime condition check code", () => {
	const fs = require("fs");
	const path = require("path");

	const source = fs.readFileSync(
		path.join(
			__dirname,
			"./shared.js"
		),
		"utf-8"
	);
	if (source.includes("__rspack_context.j")) {
		expect(source).toContain(`"a-runtime" == __rspack_context.j`)
		expect(source).toContain(`"b-runtime" == __rspack_context.j`);
		expect(source).toContain(`/^[ab]x\\-name$/.test(__rspack_context.j)`);
	} else {
		expect(source).toContain(`"a-runtime" == __webpack_require__.j`)
		expect(source).toContain(`"b-runtime" == __webpack_require__.j`);
		expect(source).toContain(`/^[ab]x\\-name$/.test(__webpack_require__.j)`);
	}
})

it("should emit one runtime condition per checked star reexport", () => {
	const fs = require("fs");
	const path = require("path");
	const source = fs.readFileSync(path.join(__dirname, "./shared.js"), "utf-8");
	const runtime = source.includes("__rspack_context.j")
		? "__rspack_context"
		: "__webpack_require__";
	const condition = `"a-runtime" == ${runtime}.j`;
	const largeModule = getGeneratedModule(source, "./checked-shared.js");
	const smallModule = getGeneratedModule(source, "./small-checked-shared.js");

	// One condition for all checked reexports.
	expect(largeModule.split(condition)).toHaveLength(2);
	expect(smallModule.split(condition)).toHaveLength(2);
	expect(largeModule.split(".forEach(")).toHaveLength(2);
	expect(smallModule.split(".forEach(")).toHaveLength(2);
	expect(largeModule).not.toContain(`${runtime}.cr(`);
	expect(smallModule).not.toContain(`${runtime}.cr(`);

	for (const asset of [
		"a-runtime.js",
		"b-runtime.js",
		"ax-name.js",
		"bx-name.js"
	]) {
		const runtimeSource = fs.readFileSync(
			path.join(__dirname, asset),
			"utf-8"
		);
		expect(runtimeSource).not.toContain(`${runtime}.cr =`);
	}
});
