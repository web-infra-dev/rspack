it("should be able to process AST from loader", function() {
	expect(require("./ast-loader.mjs!./module")).toBe("ok");
});
