import * as consumer from "./consumer";
import { value } from "./lib";

it("should update each emitted import phase after omitting lazy reexports", async () => {
	await globalThis.LAZY_BARREL_DEFER_READY;
	expect(consumer.read()).toEqual([["local", "readDeferred", "value"], 1, 1]);
	expect(value()).toBe(1);

	// Both the ordinary reexport and the deferred import must receive the update.
	await NEXT_HMR();
	expect(consumer.read()).toEqual([["local", "readDeferred", "value"], 2, 2]);
	expect(value()).toBe(2);
	expect(globalThis.LAZY_BARREL_DEFER_ACCEPTS).toBe(1);

	// Removing the namespace consumer leaves only the deferred import binding.
	await NEXT_HMR();
	await globalThis.LAZY_BARREL_DEFER_READY;
	expect(consumer.read()).toEqual([7, 2]);
	expect(globalThis.LAZY_BARREL_DEFER_ACCEPTS).toBe(0);

	await NEXT_HMR();
	expect(consumer.read()).toEqual([7, 3]);
	expect(value()).toBe(3);
	expect(globalThis.LAZY_BARREL_DEFER_ACCEPTS).toBe(1);
});

import.meta.webpackHot.accept("./consumer");
import.meta.webpackHot.accept("./lib/vanilla");
