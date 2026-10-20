import hit from "./hit";
import invalid from "./invalid";
import miss from "./miss";

it("should evaluate code filters inside a parallel loader batch", () => {
	expect(hit).toBe("hit");
	expect(miss).toBe("__VALUE__");
	expect(invalid).toBe("invalid-preserved");
});
