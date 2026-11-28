import * as values from "./module";
import { leaf } from "./leaf";
import "./side-effect";

it("should omit the dynamic import when its export becomes unused again", async () => {
	expect(values.live).toBe("live");
	expect(leaf).toBe("after");
	expect(globalThis.sideEffectValue).toBe("stable");
	expect((await values.always()).value).toBe("always");
});
