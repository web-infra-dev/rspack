it("should allow combinations of async and sync loaders with `this.async`s or `this.callback`s", function () {
	expect(require("./a?case-1")).toBe("a");
	expect(require("./a?case-2")).toBe("a");
	expect(require("./a?case-3")).toBe("a");
	expect(require("./a?case-4")).toBe("a");
	expect(require("./a?case-5")).toBe("a");
	expect(require("./a?case-6")).toBe("a");
	expect(require("./a?case-7")).toBe("a");
	expect(require("./a?case-8")).toBe("a");
	expect(require("./a?case-9")).toBe("a");
	expect(require("./a?case-10")).toBe("a");
});
