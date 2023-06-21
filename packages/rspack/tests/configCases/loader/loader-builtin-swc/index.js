interface A {
	a: number;
}

const b: A = { a: 123 };

it("expect normal conversion ts ", () => {
	expect(b).toEqual({ a: 123 });
});
