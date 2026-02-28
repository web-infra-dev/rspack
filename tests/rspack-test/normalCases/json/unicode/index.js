it("should require json via require", function () {
	expect(require("./a.json")).toEqual({
		a: "Some test message with unicode\u2027char",
		b: "Some test message with unicode\u2028char",
		c: "Some test message with unicode\u2029char",
		d: "Some test message with unicode\u2027\u2028\u2029char"
	});
	expect(require("./b.json")).toEqual({
		a: "\u2028c"
	});
	expect(require("./c.json")).toEqual("\u2028c");
});
