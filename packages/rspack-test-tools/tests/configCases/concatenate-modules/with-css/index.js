import { test } from "./lib.js";
import styles from "./foo.css";
test;

it("should concatenate css", () => {
	expect(styles).toMatchObject({
		foo: "foo_css__foo",
		test: "foo_css__test"
	});
});
