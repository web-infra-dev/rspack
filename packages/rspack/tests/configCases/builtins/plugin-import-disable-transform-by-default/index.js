import { AaBb } from "./src/foo";
import { CcDd } from "./src/bar";

it("should resolve right even with `disableTransformByDefault` is on", () => {
	expect(AaBb).toBe("Foo");
	expect(CcDd).toBe("Bar");
});
