import styles from "./foo.css";
import { test } from "./lib.js";

test;

it("should concatenate css", () => {
	expect(styles).toMatchObject({
		foo: "./foo.css__foo",
		test: "./foo.css__test"
	});
});
