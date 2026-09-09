require("./created-require");

it("initializes catch-body declarations separately from parameter defaults", () => {
	try {
		throw [];
	} catch ([value = require("./value")]) {
		const require = () => "body";
		class Local {}
		expect(value).toBe("dependency");
		expect(require("./missing-catch-body")).toBe("body");
		expect(new Local()).toBeInstanceOf(Local);
	}
});

it("preserves local declaration metadata inside nested blocks", () => {
	const value = "outer";
	{
		const value = require("./value");
		expect(value).toBe("dependency");
	}
	expect(value).toBe("outer");
});
