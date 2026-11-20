import dep from "dep";

it("should update the resolution when the package map changes in watch mode", () => {
	switch (WATCH_STEP) {
		case "0":
			expect(dep).toBe("dep@1.0.0");
			break;
		case "1":
			// The package map now selects another package for the dependency.
			expect(dep).toBe("dep@2.0.0");
			break;
		default:
			throw new Error("unexpected update");
	}
});
