import { getNamespaces } from "./consumer";
import { evaluations } from "./state";

it("should introduce and update deferred namespaces through HMR", async () => {
	expect(getNamespaces()).toEqual([]);
	await NEXT_HMR();
	const [namespace, pending] = getNamespaces();
	expect(evaluations).toEqual([]);
	expect(namespace.value).toBe(1);
	expect(namespace.value).toBe(1);
	namespace.increment();
	expect(namespace.value).toBe(2);
	expect(Object.keys(namespace).sort()).toEqual(["increment", "value"]);
	expect(evaluations).toEqual(["deferred 1"]);

	await NEXT_HMR();
	const [updated] = getNamespaces();
	expect(updated.value).toBe(10);
	updated.increment();
	expect(updated.value).toBe(11);
	// The pending proxy must retain its exports object when the helper is updated.
	expect(pending.value).toBe(42);
	expect(pending.value).toBe(42);
	expect(evaluations).toEqual(["deferred 1", "deferred 10", "pending"]);
});

module.hot.accept("./consumer");
