import value from "./value.js";

console.log(value);

it("should detect file changes after restoring the persistent cache", async () => {
	if (COMPILER_INDEX === 0) await NEXT_START();
});
