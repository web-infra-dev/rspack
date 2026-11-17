it("should resolve packages once the package map is created in watch mode", () => {
	switch (WATCH_STEP) {
		case "0":
			// The package map does not exist yet.
			expect(() => require("dep")).toThrow();
			break;
		case "1":
			expect(require("dep")).toBe("store/dep");
			break;
		default:
			throw new Error("unexpected update");
	}
});
