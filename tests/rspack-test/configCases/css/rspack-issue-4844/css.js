import * as styles from "./a.module.css";

it("css module should build success", () => {
	expect(typeof styles["foo"]).toBe("string");
});
