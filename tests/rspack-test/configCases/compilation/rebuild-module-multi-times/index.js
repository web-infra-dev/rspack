import { a as a1 } from "./a?1";
import { a as a2 } from "./a?2";
import { a as a3 } from "./a?3";

it("compilation rebuild module should works", () => {
	expect(a1 + a2 + a3).toBe(15)
});
