import * as ns from "./lib";
import { barUsed, bazUsed, fooUsed } from "./lib";

it("should track const property keys on esm namespace access", () => {
	const fooKey = "foo";
	const barKey = "bar";

	expect(ns[fooKey]).toBe("foo");
	expect(barKey in ns).toBe(true);

	expect(fooUsed).toBe(true);
	expect(barUsed).toBe(true);
	expect(bazUsed).toBe(false);
});

it("should not error when guarded static keys are missing", () => {
	const missingKey = "missing";
	const unguardedMissingKey = "unguardedMissing";

	expect(missingKey in ns).toBe(false);
	if (missingKey in ns) {
		expect(ns[missingKey]).toBe(undefined);
		expect(ns[missingKey].prop).toBe(undefined);
	}
	expect(ns[unguardedMissingKey]).toBe(undefined);
});
