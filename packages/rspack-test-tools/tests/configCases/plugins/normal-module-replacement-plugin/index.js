it("should resolve with replaced request", function () {
	expect(require("./request.v1")).toBe("v2");
	expect(require("./request.v1.js")).toBe("v2");
});

it("should use replaced resource", function () {
	expect(require("./resource.foo")).toBe("bar");
	expect(require("./resource.foo.js")).toBe("bar");
});
