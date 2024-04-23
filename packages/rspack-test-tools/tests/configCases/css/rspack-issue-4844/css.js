import styles from "./a.module.css";

it("css module should build success", () => {
	expect(typeof styles["xxx"]).toBe("string");
});
