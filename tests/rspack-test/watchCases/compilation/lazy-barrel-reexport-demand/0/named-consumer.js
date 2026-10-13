import { create, createStore } from "./lib";
import { create as namedCreate, makeStore } from "./lib/named";

it("should resolve named consumers to the defining module", () => {
	expect(create("local").value).toBe("local");
	expect(createStore("star").value).toBe("star");
	expect(namedCreate("named-local").value).toBe("named-local");
	expect(makeStore("named").value).toBe("named");
});
