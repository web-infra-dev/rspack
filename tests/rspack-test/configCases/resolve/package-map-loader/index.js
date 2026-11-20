import value from "./value";
import dep from "dep";

it("should locate loaders with the package map of resolveLoader", () => {
	expect(value).toBe("store/value-loader");
});

it("should not use the package map of resolveLoader for modules", () => {
	expect(dep).toBe("node_modules/dep");
});
