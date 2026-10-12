import load from "./loader";
import { evaluations } from "./state";

it("should introduce deferred async dependencies through HMR", async () => {
	expect(await load()).toBe("initial");
	await NEXT_HMR();
	const namespace = await load();
	expect(evaluations).toEqual(["async dependency"]);
	expect(namespace.value).toBe(42);
	expect(namespace.value).toBe(42);
	namespace.increment();
	expect(namespace.value).toBe(43);
	expect(evaluations).toEqual(["async dependency", "deferred"]);
});

module.hot.accept("./loader");
