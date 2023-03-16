class Parent {
	time = new Date().valueOf();
}

class Child extends Parent {
	time;
}

it("should child class have defined field in js", () => {
	const parent = new Parent();
	const child = new Child();
	expect(typeof parent.time).toBe("number");
	expect(typeof child.time).toBe("undefined");
});
