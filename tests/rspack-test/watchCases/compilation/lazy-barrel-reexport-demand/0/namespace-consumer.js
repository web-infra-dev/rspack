import * as star from "./lib";
import * as named from "./lib/named";

it("should expose already activated reexports to a later namespace consumer", () => {
	expect(Object.keys(star).sort()).toEqual(["create", "createStore"]);
	expect(Object.keys(named).sort()).toEqual(["create", "makeStore"]);
	expect(star.createStore("namespace").value).toBe("namespace");
	expect(named.makeStore("namespace").value).toBe("namespace");
});
