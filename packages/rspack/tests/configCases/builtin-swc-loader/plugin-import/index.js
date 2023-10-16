import { AaBb } from "./src/foo";
import { CcDd } from "./src/bar";

it("should resolve right", () => {
	expect(AaBb).toBe("Foo");
	expect(CcDd).toBe("Bar");
});
