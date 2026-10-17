import * as star from "./lib";
import * as named from "./lib/named";

// The imported modules stay the same; only their referenced exports change.
it("should restore namespace getters without changing chunk graph topology", () => {
	expect(Object.keys(star).sort()).toEqual(["create", "createStore"]);
	expect(Object.keys(named).sort()).toEqual(["create", "makeStore"]);
});
