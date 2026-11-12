import * as values from "./module";
import { leaf } from "./leaf";
import "./side-effect";

it("should omit a dynamic import owned by an unused export", async () => {
	expect(values.live).toBe("live");
	expect(leaf).toBe(WATCH_STEP === "0" ? "before" : "after");
	expect(globalThis.sideEffectValue).toBe("stable");
	expect((await values.always()).value).toBe("always");
});
