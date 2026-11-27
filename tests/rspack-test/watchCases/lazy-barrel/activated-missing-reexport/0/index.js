import { a } from "./pkg/named";

it("should not report the unrequested re-export", () => {
	expect(a).toBe("named");
});
