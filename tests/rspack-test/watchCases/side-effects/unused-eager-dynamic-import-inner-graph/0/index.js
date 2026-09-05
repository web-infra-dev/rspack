import { live } from "./module";
import { leaf } from "./leaf";

it("should omit an eager dynamic import owned by an unused export", () => {
	expect(live).toBe("live");
	expect(leaf).toBe(WATCH_STEP === "0" ? "before" : "after");
});
