import { createStore, makeStore } from "./lib";
import * as ns from "./lib";

// Only the referenced exports change; the connections and lazy dependencies stay the same.
it("should expose already activated reexports to the namespace consumer", () => {
	expect(Object.keys(ns).sort()).toEqual(["create", "createStore", "makeStore"]);
	expect(ns.createStore).toBe(createStore);
	expect(ns.makeStore).toBe(makeStore);
});
