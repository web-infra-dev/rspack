class Parent {
	time: undefined | number = new Date().valueOf();
}

class Child extends Parent {
	time: undefined | number;
}

it("should child class ignore defined field in ts", () => {
	const parent = new Parent();
	const child = new Child();
	expect(typeof parent.time).toBe("number");
	expect(typeof child.time).toBe("number");
});
