import { value as collision } from "./collision";
import { value, caught, shadowed, independent } from "./module";

it("renames linked catch and var bindings together", () => {
	expect(collision).toBe("collision");
	expect(value).toBe("outer");
	expect(caught()).toEqual({ value: "default" });
});

it("preserves independent catch and parameter environments", () => {
	expect(independent()).toBe("independent");
	expect(shadowed()).toEqual(["body", "parameter"]);
});
