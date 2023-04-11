import foo from "./foo";
import baz from "./baz";

it("should resolve 'foo.bar', byDependency '.bar' extension works", function () {
	expect(foo).toBe("bar");
});

it("should resolve 'baz.js', byDependency '...' extensions works", function () {
	expect(baz).toBe("baz");
});
