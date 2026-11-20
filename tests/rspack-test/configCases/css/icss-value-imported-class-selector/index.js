import * as shared from "./shared.module.css";
import * as reexportStyles from "./reexport.module.css";
import * as consumer from "./consumer.module.css";
import "./side-effect.module.css";
import { readFileSync } from "fs";
import { join } from "path";

it("should resolve imported @value and :import class/id selectors to the shared scope", () => {
	const isLocalIdentOnly = process.env.VARIANT === "production-minified-local";
	const sharedJsPrefix = isLocalIdentOnly ? "" : "shared.module__";
	const consumerJsPrefix = isLocalIdentOnly ? "" : "consumer.module__";
	const sharedCssPrefix = isLocalIdentOnly ? "" : "shared\\.module__";
	const consumerCssPrefix = isLocalIdentOnly ? "" : "consumer\\.module__";

	expect(shared["shared-class"]).toBe(`${sharedJsPrefix}shared-class`);
	expect(reexportStyles["shared-class"]).toBe(`${sharedJsPrefix}shared-class`);

	// Direct, aliased, multi-hop, local-chained, and :import selectors do not
	// append phantom consumer fallback classes to JS exports.
	expect(consumer["shared-class"]).toBe(`${sharedJsPrefix}shared-class`);
	expect(consumer["shared-class-alias"]).toBe(`${sharedJsPrefix}shared-class`);
	expect(consumer["local-chain"]).toBe(`${sharedJsPrefix}shared-class`);
	expect(consumer["icss-alias"]).toBe(`${sharedJsPrefix}shared-class`);
	expect(consumer["shared-id"]).toBe(`${sharedJsPrefix}shared-id`);

	// Composed classes preserve full composition in JS while keeping only the
	// target class selector in CSS.
	expect(consumer["composed-class"]).toBe(
		`${sharedJsPrefix}composed-class ${sharedJsPrefix}base`
	);

	// Escaped class names keep unescaped JS values and escaped CSS selectors.
	expect(consumer["half-alias"]).toBe(`${sharedJsPrefix}u-1/2`);
	expect(consumer["icss-half"]).toBe(`${sharedJsPrefix}u-1/2`);

	// Plain @value tokens and dashed custom properties fall back to consumer
	// local selectors instead of corrupting selectors.
	expect(consumer["plain-token"]).toBe(`12px ${consumerJsPrefix}plain-token`);
	expect(consumer["accent"]).toBe(
		`--${sharedJsPrefix}accent ${consumerJsPrefix}accent`
	);

	const css = readFileSync(join(__dirname, `bundle${__STATS_I__}.css`), "utf-8");

	expect(css).toContain(`.${sharedCssPrefix}shared-class`);
	expect(css).toContain(`article .${sharedCssPrefix}shared-class`);
	expect(css).toContain(`#${sharedCssPrefix}shared-id`);
	expect(css).toContain(`.${sharedCssPrefix}composed-class`);
	expect(css).not.toContain(
		`.${sharedCssPrefix}composed-class ${sharedCssPrefix}base`
	);
	expect(css).toContain(`.${sharedCssPrefix}u-1\\/2`);
	expect(css).toContain(`.${consumerCssPrefix}plain-token`);
	expect(css).toContain(`.${consumerCssPrefix}accent`);
	expect(css).not.toContain(".--");
	expect(css).toContain(".real-global-btn");

	if (!isLocalIdentOnly) {
		expect(css).not.toContain(".reexport\\.module__shared-class");
		expect(css).not.toContain(".consumer\\.module__shared-class");
		expect(css).not.toContain(".consumer\\.module__shared-class-alias");
		expect(css).not.toContain(".consumer\\.module__local-chain");
		expect(css).not.toContain(".consumer\\.module__composed-class");
		expect(css).not.toContain(".consumer\\.module__half-alias");
		expect(css).not.toContain(".consumer\\.module__icss-alias");
		expect(css).not.toContain(".consumer\\.module__icss-half");
		expect(css).not.toContain("#consumer\\.module__shared-id");
		expect(css).toContain(".consumer\\.module__unused-plain");
	} else {
		// Unused plain @value fallback selectors are tree-shaken by
		// removeUnusedLocalIdents, while side-effect overrides of used shared
		// classes are preserved even when localIdentName is "[local]".
		expect(css).not.toContain("unused-plain");
		expect(css).toMatch(/font-weight:\s*(700|bold)/);
	}
});
