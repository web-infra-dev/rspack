import { a } from "./pkg";

it("should keep the unrequested re-export of a side-effect-free barrel lazy", () => {
	expect(a).toBe("a");
});
