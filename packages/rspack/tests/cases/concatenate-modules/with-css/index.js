import { test } from "./lib.js";
import * as styles from "./foo.css";
test;

it("should use exports info per runtime ", () => {
	expect(styles).toEqual({
		foo: "_59940e09e3068aa3af1f",
		test: "c9988779a2963d645d5a"
	});
});
