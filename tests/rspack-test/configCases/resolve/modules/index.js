import a from "foo/a";
import b from "foo/b";

it("resolve modules should work fine", async () => {
	expect(a).toEqual("a");
	expect(b).toEqual("b");
});
