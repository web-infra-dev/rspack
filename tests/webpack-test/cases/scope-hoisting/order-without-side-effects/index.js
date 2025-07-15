import { a } from "./a";
import { b } from "./b";
import { array } from "./tracker";

it("should concatenate in correct order", function() {
	expect(b).toBe(2);
	expect(a).toBe(1);
	expect(array).toEqual(["a", "b"]);
});
