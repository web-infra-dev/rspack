import data1 from "./array.json?1";
import data2 from "./array.json?2";
import data3 from "./reexport-imported";

it("should allow to call prototype methods", () => {
	expect(data1.map(d => d * 2)).toEqual([2, 2, 4, 6, 10]);
	expect(data2.map(d => d * 2)).toEqual([2, 2, 4, 6, 10]);
	expect(require("./array.json?2").map(d => d * 2)).toEqual([2, 2, 4, 6, 10]);
	expect(require("./array.json?3").map(d => d * 2)).toEqual([2, 2, 4, 6, 10]);
	try {
		Object.prototype.smoosh = function () {
			return this.map(d => d + 1);
		};
		expect(data1.smoosh()).toEqual([2, 2, 3, 4, 6]);
		expect(data2.smoosh()).toEqual([2, 2, 3, 4, 6]);
		expect(require("./array.json?2").smoosh()).toEqual([2, 2, 3, 4, 6]);
		expect(require("./array.json?3").smoosh()).toEqual([2, 2, 3, 4, 6]);
	} finally {
		delete Object.prototype.smoosh;
	}
});

it("should allow to call prototype methods on an imported CommonJS reexport", () => {
	expect(data3.map(d => d * 2)).toEqual([2, 2, 4, 6, 10]);
});

it("should allow to call prototype methods on a required CommonJS reexport", () => {
	expect(require("./reexport-required").map(d => d * 2)).toEqual([2, 2, 4, 6, 10]);
});

it("should allow to call prototype methods on a dynamically imported CommonJS reexport", async () => {
	await import("./reexport-dynamic").then(m => {
		expect(m.default.map(d => d * 2)).toEqual([2, 2, 4, 6, 10]);
	});
});

it("should allow to call prototype methods on an assigned CommonJS require", () => {
	const data = require("./reexport-assigned");
	expect(data.map(d => d * 2)).toEqual([2, 2, 4, 6, 10]);
});

it("should keep object keys when calling a prototype method on a CommonJS reexport", () => {
	expect(require("./reexport-object").hasOwnProperty("array")).toBe(true);
});

it("should keep nested JSON values when calling prototype methods on a CommonJS reexport", () => {
	const data = require("./reexport-nested");
	expect(data.array.map(d => d * 2)).toEqual([2, 2, 4, 6, 10]);
	expect(data.object.hasOwnProperty("kept")).toBe(true);
});
