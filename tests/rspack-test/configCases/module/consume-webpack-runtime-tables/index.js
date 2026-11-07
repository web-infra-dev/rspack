import value, { moduleIds } from "./lib.js";
import dependency from "./dependency.js";

it("should keep a nested bundle's runtime tables separate from the chunk's own", () => {
	expect(moduleIds).toEqual(["940"]);
	expect(value).toBe(42);
	expect(dependency).toBe("dependency");
});
