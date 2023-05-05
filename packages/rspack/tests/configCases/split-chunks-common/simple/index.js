it("should run", function () {
	var a = require("./a");
	expect(a).toBe("a");
});

// TODO: Rspack doesn't support `require.main`
// it("should be main", function () {
// 	expect(require.main).toBe(module);
// });
