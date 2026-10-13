import hit from "./hit";
import miss from "./miss";

it("should evaluate code filters inside a parallel loader batch", () => {
	expect(hit).toBe("hit");
	expect(miss).toBe("__VALUE__");
});
