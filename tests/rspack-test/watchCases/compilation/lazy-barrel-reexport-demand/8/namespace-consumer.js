import * as star from "./lib";
import * as named from "./lib/named";

it("should add namespace getters again after rebuilding the barrel", () => {
	expect(Object.keys(star).sort()).toEqual(["create", "createStore", "revision"]);
	expect(Object.keys(named).sort()).toEqual(["create", "makeStore", "revision"]);
	expect(star.createStore("namespace").value).toBe("namespace");
	expect(named.makeStore("namespace").value).toBe("namespace");
});
