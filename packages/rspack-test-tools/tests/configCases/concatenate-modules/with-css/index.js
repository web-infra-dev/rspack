import { test } from "./lib.js";
import styles from "./foo.css";
test;

it("should concatenate css", () => {
	expect(styles).toMatchObject({
		foo: "__foo_css__foo",
		test: "__foo_css__test"
	});
});
