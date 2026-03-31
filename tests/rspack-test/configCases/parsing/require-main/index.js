it("should define require.main", function() {
	expect(require.main).toBe(module);
});

it("should handle require.main from another module", async () => {
	const { main } = require("./module");
	expect(main).toBe(false);
});
