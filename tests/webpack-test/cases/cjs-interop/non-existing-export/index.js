import { named as named2 } from "./esModule";
import { named } from "./module";

it("should emit errors", () => {
	expect(named).toBe(undefined);
	expect(named2).toBe(undefined);
});
