function clsDecorator(cls) {
	cls.prototype.a = 1;
}
@clsDecorator
class A {}

function methodDecorator(target, name, descriptor) {
	const fn = descriptor.value;
	descriptor.value = function (...args) {
		const res = fn.call(this, ...args);
		if (res === undefined) {
			return 2;
		}
		return res;
	};
}
class B {
	@methodDecorator
	test_return_2() {}

	@methodDecorator
	test_return_3() {
		return 3;
	}
}

it("should transform decorator in TypeScript file even with `disableTransformByDefault` is on", () => {
	const a = new A();
	// @ts-expect-error
	expect(a.a).toBe(1);
	const b = new B();
	expect(b.test_return_2()).toBe(2);
	expect(b.test_return_3()).toBe(3);
});
