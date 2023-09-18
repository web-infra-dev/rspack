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
export { B };
