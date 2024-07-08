import yes from "./fully-specified.mjs";

it("should error when not fullySpecified for mjs", () => {
	expect(yes).toBe("fully-specified");
	let count = 0;
	try {
		require("./not-fully-specified");
	} catch (e) {
		count += 1;
		expect(
			e.message.includes("Cannot find module './not-fully-specified'")
		).toBe(true);
	}
	expect(count).toBe(1);
});
