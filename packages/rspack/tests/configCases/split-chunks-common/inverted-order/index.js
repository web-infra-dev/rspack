var a = require("./a");

it("should run", function () {
	expect(a).toBe("a");
});

// TODO: Rspack doesn't support `require.main`
// var mainModule = require.main;

// it("should be main", function () {
// 	expect(mainModule).toBe(module);
// });
