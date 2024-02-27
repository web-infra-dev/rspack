class A {
	constructor(
		func = () => {
			this.a = 1;
		}
	) {
		func();
	}
}

it("ensure top level is right in constructor parameters", () => {
	const a = new A();
	expect(a.a).toBe(1);
	expect(typeof a).toBe("object");
});

export default "";
