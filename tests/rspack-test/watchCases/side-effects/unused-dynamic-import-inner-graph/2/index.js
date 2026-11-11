import * as values from "./module";
import { leaf } from "./leaf";
import "./side-effect";

it("should restore the dynamic import when its export becomes used", async () => {
	expect(values.live).toBe("live");
	expect(leaf).toBe("after");
	expect(globalThis.sideEffectValue).toBe("stable");
	expect((await values.always()).value).toBe("always");
	expect((await values.feature()).value).toBe("feature");
});
