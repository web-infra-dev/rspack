import { value } from "./module";

it("should preserve placeholder-looking text in a string", () => {
	expect(DEFINED_STRING).toBe("__rspack_module_ref0__._");
	expect(value).toBe(42);
});
