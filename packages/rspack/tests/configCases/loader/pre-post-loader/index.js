// TODO: use inline loader
// it("should apply pre and post loaders correctly", function() {
// 	expect(require("./a")).toBe("resource loader2 loader1 loader3");
// 	expect(require("!./a")).toBe("resource loader2 loader3");
// 	expect(require("!!./a")).toBe("resource");
// 	expect(require("-!./a")).toBe("resource loader3");
// });

it("should apply pre and post loaders correctly", function () {
	expect(require("./a?t0")).toBe("resource loader2 loader1 loader3");
	expect(require("./a?t1")).toBe("resource loader2 loader3");
	expect(require("./a?t2")).toBe("resource");
	expect(require("./a?t3")).toBe("resource loader3");
});
