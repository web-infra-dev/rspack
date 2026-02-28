import { lib } from "lib";
import * as reexports from "./reexports";

it("should use correct resolve options", () => {
	expect(lib).toBe("lib");
	expect(reexports.lib).toBe("lib2");
})