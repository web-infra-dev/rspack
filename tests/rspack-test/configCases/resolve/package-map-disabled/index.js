import dep from "dep";
import legacy from "./legacy";

it("should use the package map of the resolve options", () => {
	expect(dep).toBe("store/dep");
});

it("should not use the package map for a rule that disables it", () => {
	expect(legacy).toBe("node_modules/dep");
});
