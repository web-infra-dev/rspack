import * as consumer from "./consumer";
import { value } from "./lib";

it("should update only the import bindings left after omitting lazy reexports", async () => {
	await Promise.all([globalThis.LAZY_BARREL_READY, globalThis.LAZY_BARREL_DEFER_READY]);
	expect(consumer.read()).toEqual([
		["local", "value"], ["local", "readDeferred", "value"], 1, 1, 1
	]);
	expect(value()).toBe(1);

	// Update both ordinary and deferred bindings while all getters are present.
	await NEXT_HMR();
	expect(consumer.read()).toEqual([
		["local", "value"], ["local", "readDeferred", "value"], 2, 2, 2
	]);
	expect(value()).toBe(2);
	expect(globalThis.LAZY_BARREL_ACCEPTS).toBe(1);
	expect(globalThis.LAZY_BARREL_DEFER_ACCEPTS).toBe(1);

	// The first barrel loses its binding; the second keeps only its deferred binding.
	await NEXT_HMR();
	await Promise.all([globalThis.LAZY_BARREL_READY, globalThis.LAZY_BARREL_DEFER_READY]);
	expect(consumer.read()).toEqual([7, 7, 2]);
	expect(globalThis.LAZY_BARREL_ACCEPTS).toBe(0);
	expect(globalThis.LAZY_BARREL_DEFER_ACCEPTS).toBe(0);

	await NEXT_HMR();
	expect(consumer.read()).toEqual([7, 7, 3]);
	expect(value()).toBe(3);
	expect(globalThis.LAZY_BARREL_ACCEPTS).toBe(1);
	expect(globalThis.LAZY_BARREL_DEFER_ACCEPTS).toBe(1);
});

module.hot.accept("./consumer");
module.hot.accept("./lib/vanilla");
