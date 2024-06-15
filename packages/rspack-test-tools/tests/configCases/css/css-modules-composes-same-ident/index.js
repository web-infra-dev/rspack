import * as styles from "./index.module.css";

it("should generate correct exports", () => {
	expect(styles).toEqual(nsObj({
		"input": "./index.module-input ./input.module-input",
	}))
})