import * as consumer from "./consumer";
import { value } from "./lib";

it("should accept updates after omitting lazy reexports and rebuilding the barrel", async () => {
	await globalThis.LAZY_BARREL_READY;
	expect(consumer.read()).toEqual(["local", "value"]);
	expect(value()).toBe(1);
	await NEXT_HMR();
	await globalThis.LAZY_BARREL_READY;
	expect(consumer.read()).toEqual([7]);
	expect(globalThis.LAZY_BARREL_ACCEPTS).toBe(0);
	await NEXT_HMR();
	expect(value()).toBe(2);
	expect(globalThis.LAZY_BARREL_ACCEPTS).toBe(1);
	await NEXT_HMR();
	await globalThis.LAZY_BARREL_READY;
	expect(consumer.read()).toEqual([7]);
	expect(globalThis.LAZY_BARREL_ACCEPTS).toBe(0);
	await NEXT_HMR();
	expect(value()).toBe(3);
	expect(globalThis.LAZY_BARREL_ACCEPTS).toBe(1);
});

module.hot.accept("./consumer");
module.hot.accept("./lib/vanilla");
