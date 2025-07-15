import { b } from "./module";
import array from "./tracker";

it("should evaluate modules in the correct order", () => {
	expect(b).toEqual("b");
	expect(array).toEqual(["b", "a"]);
})
