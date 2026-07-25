import { DecoratedA } from "./model-a";
import { DecoratedB } from "./model-b";

it("should share TypeScript decorator fallbacks", () => {
	expect(new DecoratedA().value).toBe("a");
	expect(new DecoratedB().value).toBe("b");
});
