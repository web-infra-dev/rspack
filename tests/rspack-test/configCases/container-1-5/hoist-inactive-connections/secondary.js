import { getRuntimeSpecificValue } from "./runtime-plugin";

it("keeps a dependency used by only one runtime loadable", () => {
	expect(getRuntimeSpecificValue()).toBe("runtime-specific-pkg export");
});
