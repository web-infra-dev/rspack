it("should correctly export stuff from not parsed modules", function () {
	expect(require("./not-parsed-a")).toBe("ok");
});
