import * as values from "./at-rule-value.module.css";
import * as references from "./references.module.css";
import { readFileSync } from "fs";
import { join } from "path";

it("should substitute webpack's @value media queries and selectors", () => {
	const css = readFileSync(join(__dirname, `bundle${__STATS_I__}.css`), "utf-8");
	expect(values.small).toBe("(max-width: 599px)");
	expect(values["colorValue-v3"]).toBe(".red");
	expect(css).toContain("@media (max-width: 599px) {");
	expect(css).toContain(".red {\n\tcolor: .red;");
	expect(css).not.toContain("@media small");
	expect(css).not.toContain("colorValue-v3 {");
});

it("should substitute references throughout selector and at-rule preludes", () => {
	const css = readFileSync(join(__dirname, `bundle${__STATS_I__}.css`), "utf-8");
	expect(references.selector).toBe(".globalThing");
	expect(css).toContain(".globalThing { color: blue; }");
	expect(css).toContain("article .globalThing, section > .globalThing {");
	expect(css).toContain(":is(.globalThing) {");
	expect(css).toContain("& .globalThing {");
	expect(css).toContain("@media screen and (max-width: 599px) {");
	expect(css).toContain("@supports (display: grid) {");
	expect(css).toContain("article other-selector-value, section > reexport-selector-value {");
	expect(css).toContain("selector: blue;");
	expect(css).toContain('content: "selector mq";');
});
