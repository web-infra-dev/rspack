it("should allow combinations of async and sync loaders", function() {
	expect(require("./loaders/syncloader.mjs!./a")).toBe("a");
	expect(require("./loaders/asyncloader.mjs!./a")).toBe("a");

	expect(require("./loaders/syncloader.mjs!./loaders/syncloader.mjs!./a")).toBe("a");
	expect(require("./loaders/syncloader.mjs!./loaders/asyncloader.mjs!./a")).toBe("a");
	expect(require("./loaders/asyncloader.mjs!./loaders/syncloader.mjs!./a")).toBe("a");
	expect(require("./loaders/asyncloader.mjs!./loaders/asyncloader.mjs!./a")).toBe("a");

	expect(require("./loaders/asyncloader.mjs!./loaders/asyncloader.mjs!./loaders/asyncloader.mjs!./a")).toBe("a");
	expect(require("./loaders/asyncloader.mjs!./loaders/syncloader.mjs!./loaders/asyncloader.mjs!./a")).toBe("a");
	expect(require("./loaders/syncloader.mjs!./loaders/asyncloader.mjs!./loaders/syncloader.mjs!./a")).toBe("a");
	expect(require("./loaders/syncloader.mjs!./loaders/syncloader.mjs!./loaders/syncloader.mjs!./a")).toBe("a");
});
