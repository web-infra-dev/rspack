import one from "./pkg-1.json" with { type: "json" };
import two from "./pkg-2.json" with { type: "json" };
import three from "./pkg-3.json" with { type: "json" };
import four from "./pkg-4.json" with { type: "json" };

it("import attributes should work", function() {
	expect(one.type).toEqual("with");
	expect(two.type).toEqual("with");
	expect(three.type).toEqual("with");
	expect(four.type).toEqual("with");
});

