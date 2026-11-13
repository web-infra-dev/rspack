import * as styles from "./style.css";

it("should invalidate cached CSS exports when generator options change", async () => {
	expect(Object.keys(styles)).toEqual([
		COMPILER_INDEX === 0 ? "fooBar" : "foo-bar"
	]);
	if (COMPILER_INDEX === 0) await NEXT_START();
});
