import { loadOther } from "./infrastructure";

it("should keep an unchanged import owner through lazy activation", async () => {
	const ensure = __webpack_require__.eb;
	expect(ensure).toBeTypeOf("function");
	const feature = import("./feature");
	await new Promise((resolve) => setTimeout(resolve, 1000));
	await NEXT_HMR();
	expect((await feature).default).toBe(42);
	expect(await loadOther()).toBe(42);
	expect(__webpack_require__.eb).not.toBe(ensure);
	// A real unaccepted owner edit must still reject instead of hiding an unsafe update.
	await expect(NEXT_HMR()).rejects.toThrow("not accepted");
	expect(module.hot.status()).toBe("abort");
});
