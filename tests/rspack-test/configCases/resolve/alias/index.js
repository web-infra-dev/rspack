import value from "@b";
import value2 from "xx";
import value3 from "alias";
import value4 from "ignored";
import * as noAlias from "./no-alias";

it("alias should work", () => {
	expect(value).toBe("a");
	expect(value2).toBe("a");
	expect(value3).toBe("a");
	expect(value4).toStrictEqual({});
	expect(noAlias.b).toBe("@b");
});
