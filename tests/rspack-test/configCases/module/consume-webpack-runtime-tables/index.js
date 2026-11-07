import value, { moduleIds } from "./lib.js";
import dependency from "./dependency.js";

it("keeps nested runtime tables separate when concatenating modules", () => {
	expect(moduleIds).toEqual(["940"]);
	expect(value).toBe(42);
	expect(dependency).toBe("dependency");
});
