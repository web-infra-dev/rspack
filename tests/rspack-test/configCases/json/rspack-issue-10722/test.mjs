import * as foo from "./bundle0.mjs";

it("should work", () => {
	expect(foo).toMatchObject({
		"A.z": 1,
		"a+Z": 2,
		"A-Z": 3,
		await: 4,
		"😄": 5,
		9: 6
	});
});
