class Parent {
	time: undefined | number = new Date().valueOf();
}

class Child extends Parent {
	time: undefined | number;
}

it("should class use defined field", () => {
	const parent = new Parent();
	const child = new Child();
	expect(typeof parent.time).toBe("number");
	expect(typeof child.time).toBe("number");
});
