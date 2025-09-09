// import value from "./value.js";
import a from "./a.js";

// it("should exports node mode", function () {
// 	expect(value.value).toBe(1);
// });

it("should exports babel mode", function () {
	expect(a).toBe('a');
});