import { create, createStore } from "./lib";
import { create as namedCreate, makeStore } from "./lib/named";
import "./namespace-consumer";

it("should activate reexports through redirected named imports", () => {
	expect(create("local").value).toBe("local");
	expect(createStore("star").value).toBe("star");
	expect(namedCreate("named-local").value).toBe("named-local");
	expect(makeStore("named").value).toBe("named");
});
