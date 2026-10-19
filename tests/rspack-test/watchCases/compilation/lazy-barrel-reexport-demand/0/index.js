import { createStore, makeStore } from "./lib";
import * as ns from "./lib";

it("should activate reexports through redirected named imports", () => {
	expect(createStore("star").value).toBe("star");
	expect(makeStore("named").value).toBe("named");
	expect(ns.create("local").value).toBe("local");
});
