import * as styles from "./index.module.css";

it("should generate correct exports", () => {
	expect(styles).toEqual(
		nsObj({
			a: '"aaa" 123',
			b: "multiple lines  bbb"
		})
	);
});
