import * as shared from "./shared.css";
import * as consumer from "./consumer.css";
import { readFileSync } from "fs";
import { join } from "path";

it("should resolve class selectors imported through @value to the shared module's scoped class", () => {
	const css = readFileSync(join(__dirname, `bundle${__STATS_I__}.css`), "utf-8");
	const sharedClass = shared["shared-class"];
	expect(sharedClass).toBe("shared__shared-class");
	// The imported value (including the aliased form) resolves to the shared class.
	expect(consumer["shared-class"]).toContain(sharedClass);
	expect(consumer["shared-class-alias"]).toContain(sharedClass);
	// Both selectors in the consumer use the shared scoped class.
	expect(css).toContain(`.${sharedClass} {\n\tcolor: rebeccapurple;\n}`);
	expect(css).toContain(`.${sharedClass} {\n\tcolor: green;\n}`);
	expect(css).toContain(`article .${sharedClass} {\n\tcolor: blue;\n}`);
	// The consumer-local fallback must not replace the imported class.
	expect(css).not.toContain(".consumer__shared-class");
});
