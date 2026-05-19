import "./impure";
import { log } from "./tracker";

it("should preserve modules that were already impure before deferred pure checks", () => {
	expect(log).toEqual(["impure"]);
});
