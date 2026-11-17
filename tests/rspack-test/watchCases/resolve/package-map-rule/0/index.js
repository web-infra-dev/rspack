import mapped from "./mapped";
import unmapped from "dep";

it("should update the resolution when the package map of a rule or a dependency type changes in watch mode", () => {
	// Only the rule and the dependency type use the package map.
	expect(unmapped).toBe("node_modules/dep");
	switch (WATCH_STEP) {
		case "0":
			expect(mapped).toBe("dep@1.0.0");
			expect(require("dep").default).toBe("dep@1.0.0");
			break;
		case "1":
			// The package map now selects another package for the dependency.
			expect(mapped).toBe("dep@2.0.0");
			expect(require("dep").default).toBe("dep@2.0.0");
			break;
		default:
			throw new Error("unexpected update");
	}
});
