import cjs from "./cjs.js?mjs";
import cjsDynamicNo from "./cjs-dynamic.js?mjs-no";
import cjsDynamicYes from "./cjs-dynamic.js?mjs-yes";
import cjsFlagged from "./cjs-flagged.js?mjs";

it("should return correct exports when default is imported", () => {
	expect(cjs).toEqual({
		abc: "abc",
		default: "default"
	});
	expect(cjsFlagged).toEqual({
		__esModule: true,
		abc: "abc",
		default: "default"
	});
	expect(cjsDynamicYes).toMatchObject({
		__esModule: true,
		abc: "abc",
		default: "default"
	});
	expect(cjsDynamicNo).toMatchObject({
		abc: "abc",
		default: "default"
	});
});
