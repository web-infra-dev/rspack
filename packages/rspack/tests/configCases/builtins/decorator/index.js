import { B } from "./foo";

function clsDecorator(cls) {
	cls.prototype.a = 1;
}
@clsDecorator
class A {}

it("should decorator be transformed", () => {
	const a = new A();
	expect(a.a).toBe(1);
	const b = new B();
	expect(b.test_return_2()).toBe(2);
	expect(b.test_return_3()).toBe(3);
});
