const chooseObject = globalThis.__rspack_test_choose_object__ !== false;
const choosePrimitive = globalThis.__rspack_test_choose_primitive__ === true;
const __rspack_require_value__ = "user binding";

const direct = require("./object.cjs");
const conditional = require(
	chooseObject ? "./object.cjs" : "./other-object.cjs",
);
const conditionalOther = require(
	choosePrimitive ? "./object.cjs" : "./other-object.cjs",
);
const constructedObject = new require("./object.cjs");
const constructedPrimitive = new require("./primitive.cjs");
const constructedConditionalObject = new require(
	chooseObject ? "./object.cjs" : "./other-object.cjs",
);
const constructedConditionalPrimitive = new require(
	choosePrimitive ? "./object.cjs" : "./primitive.cjs",
);

it("should relocate direct and conditional CommonJS requires", () => {
	expect(__rspack_require_value__).toBe("user binding");
	expect(direct).toEqual({ value: 45 });
	expect(conditional).toBe(direct);
	expect(conditionalOther).toEqual({ value: 46 });
});

it("should preserve new require constructor return semantics", () => {
	expect(constructedObject).toBe(direct);
	expect(constructedConditionalObject).toBe(direct);
	expect(constructedPrimitive).toEqual({});
	expect(constructedConditionalPrimitive).toEqual({});
});
