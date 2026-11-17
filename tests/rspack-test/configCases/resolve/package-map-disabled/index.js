import dep from "dep";
import legacy from "./legacy";

it("should use the package map of the resolve options", () => {
	expect(dep).toBe("store/dep");
});

it("should not use the package map for a dependency type that disables it", () => {
	expect(require("dep").default).toBe("node_modules/dep");
});

it("should not use the package map for a rule that disables it", () => {
	expect(legacy).toBe("node_modules/dep");
});
