import data from "./data.txt";

it("should return the raw data if `rule.type` is st to `asset/source`", () => {
	expect(data).toBe("- Isn't Rspack a gamechanging bundler?\n  - Hella yeah!");
});
