import { c } from "./re-export";

it("should keep deferred impure pure-expression usage in rsdoctor graph", () => {
	expect(c).toBeUndefined();
});
