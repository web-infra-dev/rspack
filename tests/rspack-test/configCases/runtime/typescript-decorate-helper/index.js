const { DecoratedA } = require("./model-a");
const { DecoratedB } = require("./model-b");

it("should share TypeScript decorator fallbacks", () => {
	expect(new DecoratedA().value).toBe("a");
	expect(new DecoratedB().value).toBe("b");
});
