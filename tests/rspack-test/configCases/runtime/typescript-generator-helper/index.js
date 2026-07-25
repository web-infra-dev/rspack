const { createGenerator: createGeneratorA } = require("./module-a");
const { createGenerator: createGeneratorB } = require("./module-b");

it("should share TypeScript generator fallbacks", () => {
	const generatorA = createGeneratorA();
	expect(generatorA.next()).toEqual({ value: "a", done: false });
	expect(generatorA.next()).toEqual({ value: "done-a", done: true });

	const generatorB = createGeneratorB();
	expect(generatorB.next()).toEqual({ value: "b", done: false });
	expect(generatorB.next()).toEqual({ value: "done-b", done: true });
});
