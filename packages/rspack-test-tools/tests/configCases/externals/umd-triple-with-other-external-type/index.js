import pkg from "pkg";
import value from "./concated";

it("should works", function () {
	expect(pkg).toBe(undefined);
	expect(value).toBe(42);
});
