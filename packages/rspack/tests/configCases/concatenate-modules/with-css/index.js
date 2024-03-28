import { test } from "./lib.js";
import styles from "./foo.css";
test;

it("should concatenate css", () => {
	expect(styles).toMatchObject({
		foo: "foo-css__foo",
		test: "foo-css__test"
	});
});
