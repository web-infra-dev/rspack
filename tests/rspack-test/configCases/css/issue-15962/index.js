import * as styles from "./style.modules.css";
import * as globalStyles from "./style.global.css";
import { small, smallSecond } from "./small.modules.css";

it("localizes type-first selectors in nested rules", () => {
	const fs = require("fs");
	const path = require("path");
	const css = fs.readFileSync(path.join(__dirname, "bundle0.css"), "utf-8");
	const source = fs.readFileSync(
		path.join(__TEST_SOURCE_PATH__, "style.modules.css"),
		"utf-8"
	);
	const locals = [
		"root",
		"hovered",
		"descendant",
		"first",
		"second",
		"explicit",
		"pseudo",
		"functional",
		"identifier",
		"attribute",
		"commented",
		"namespaced",
		"escaped",
		"unicode",
		"after",
		"supports",
		"container",
		"layer",
		"outer",
		"inner",
		"deep",
		"nestedAfter",
		"nestedPseudo",
		"nestedLocal",
		"nestedMedia"
	];
	let expected = source;
	for (const name of locals) {
		expect(styles[name]).toBe(`LOCAL-${name}`);
		expected = expected.replaceAll(
			new RegExp(`([.#])${name}\\b`, "g"),
			`$1LOCAL-${name}`
		);
	}
	expected = expected
		.replaceAll("--custom", "--LOCAL-custom")
		.replaceAll(":global(.plain)", ".plain")
		.replaceAll(":local(.LOCAL-explicit)", ".LOCAL-explicit")
		.replaceAll(":global ", "")
		.replaceAll(":local ", "");
	expect(css).toContain(expected);
	expect(Object.keys(styles)).not.toContain("plain");
	expect(Object.keys(styles)).not.toContain("global");
});

it("localizes explicit locals in type-first rules in global mode", () => {
	const fs = require("fs");
	const path = require("path");
	const css = fs.readFileSync(path.join(__dirname, "bundle0.css"), "utf-8");
	expect(globalStyles.globalModeLocal).toBe("LOCAL-globalModeLocal");
	expect(globalStyles.globalModeId).toBe("LOCAL-globalModeId");
	expect(Object.keys(globalStyles).sort()).toEqual([
		"globalModeId",
		"globalModeLocal"
	]);
	expect(css).toContain("button.raw.LOCAL-globalModeLocal { color: blue; }");
	expect(css).toContain("button#LOCAL-globalModeId { color: blue; }");
	expect(css).toContain("button.rawAfter { color: green; }");
});

it("localizes type-first rules in short stylesheets", () => {
	const fs = require("fs");
	const path = require("path");
	const css = fs.readFileSync(path.join(__dirname, "bundle0.css"), "utf-8");
	expect(small).toBe("LOCAL-small");
	expect(smallSecond).toBe("LOCAL-smallSecond");
	expect(css).toContain(
		"@media screen { button.LOCAL-small, .LOCAL-smallSecond { color:red } }"
	);
});
