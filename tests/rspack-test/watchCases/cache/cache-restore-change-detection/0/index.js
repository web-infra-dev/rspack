import value from "./value.js";

it("should detect file changes after restoring the persistent cache", () => {
	expect(value).toBe(
		WATCH_STEP === "0" ? "initial value" : "updated value from watch"
	);
});
