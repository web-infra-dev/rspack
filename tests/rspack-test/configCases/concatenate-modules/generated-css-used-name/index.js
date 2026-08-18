import { __webpack_require__ as localRequire } from "./local";
import styles from "./style.css";

it("should preserve runtime globals referenced by generated CSS modules", () => {
	expect(localRequire()).toBe("local");
	expect(styles["class-name"]).toBe("class-name");
});
