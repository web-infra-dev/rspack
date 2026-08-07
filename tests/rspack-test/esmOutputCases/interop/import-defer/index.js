import { events, reset } from "./state";
import defer * as deferredEsm from "./deferred-esm";
import defer * as deferredCjs from "./deferred-cjs.cjs";
import defer * as deferredAsyncParent from "./deferred-async-parent";
import { deferredReexport } from "./reexport";

it("should preserve lazy evaluation without a module dispatcher", async () => {
	expect(events).toEqual(["async-dependency"]);
	expect(Object.prototype.toString.call(deferredEsm)).toBe(
		"[object Deferred Module]",
	);
	expect(events).toEqual(["async-dependency"]);

	expect(deferredEsm.value).toBe(1);
	expect(deferredCjs.value).toBe(2);
	expect(deferredAsyncParent.value).toBe(3);
	expect(deferredReexport.value).toBe(8);
	expect(events).toEqual([
		"async-dependency",
		"esm",
		"cjs",
		"async-parent",
		"reexport",
	]);

	reset();
	const dynamic = await import.defer("./dynamic");
	expect(events).toEqual([]);
	expect(dynamic.value).toBe(4);
	expect(events).toEqual(["dynamic"]);

	reset();
	const dynamicAsync = await import.defer("./dynamic-async-parent");
	expect(events).toEqual(["dynamic-async-dependency"]);
	expect(dynamicAsync.value).toBe(6);
	expect(events).toEqual([
		"dynamic-async-dependency",
		"dynamic-async-parent",
	]);

	reset();
	const dynamicOwnTla = await import.defer("./dynamic-own-tla");
	expect(events).toEqual(["dynamic-own-tla"]);
	expect(dynamicOwnTla.value).toBe(10);

	reset();
	const request = "a";
	const context = await import.defer(`./context/${request}`);
	expect(events).toEqual([]);
	expect(context.value).toBe(5);
	expect(events).toEqual(["context"]);

	reset();
	const asyncRequest = "async-parent";
	const asyncContext = await import.defer(`./context/${asyncRequest}`);
	expect(events).toEqual(["context-async-dependency"]);
	expect(asyncContext.value).toBe(7);
	expect(events).toEqual([
		"context-async-dependency",
		"context-async-parent",
	]);

	reset();
	const ownTlaRequest = "own-tla";
	const ownTlaContext = await import.defer(`./context/${ownTlaRequest}`);
	expect(events).toEqual(["context-own-tla"]);
	expect(ownTlaContext.value).toBe(11);

	const loadExternal = () => import.defer("./deferred-external");
	expect(typeof loadExternal).toBe("function");
});
