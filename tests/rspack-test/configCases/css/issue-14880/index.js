import * as styles from "./style.module.css";

const classFor = key => styles[key];

it("should materialize the namespace object for concatenated CSS modules", () => {
	expect(styles.button).toBe("button");
	expect(classFor("sizeSmall")).toBe("sizeSmall");
});
