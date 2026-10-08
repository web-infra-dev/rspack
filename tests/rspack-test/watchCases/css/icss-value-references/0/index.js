import { readFileSync } from "fs";
import { join } from "path";
import { value, composed } from "./style.module.css";
import { forward } from "./colors.module.css";
import { tone } from "./colors-more.module.css";
import { shadow } from "./shadow.module.css";

// The color fixtures are copied from webpack's css/reexport case.
it("should update repeated ICSS values when a transitive import changes", () => {
	const expected = WATCH_STEP === "0" ? "#e74c3c" : "#123456";
	const css = readFileSync(join(__dirname, "style.css"), "utf-8");
	for (const property of ["color", "border-color", "background"]) {
		expect(css).toContain(`${property}: ${expected};`);
	}
	expect(value).toBe(expected);
	expect(shadow).toBe(`0 0 ${expected}, 0 0 ${expected}`);
	expect(css).toContain(`box-shadow: ${shadow};`);
});

it("should update hashed class names through transitive compositions", () => {
	const expected = WATCH_STEP === "0" ? "#e74c3c" : "#123456";
	const css = readFileSync(join(__dirname, "style.css"), "utf-8");
	const ownClass = composed.split(" ")[0];
	expect(tone).toMatch(/^tone-[a-f0-9]{8}$/);
	expect(forward).toMatch(/^forward-[a-f0-9]{8} /);
	expect(composed).toBe(`${ownClass} ${forward}`);
	expect(forward.split(" ").slice(1)).toEqual([tone]);
	expect(css).toContain(`.${ownClass} {`);
	expect(css).toContain(`.${tone} {`);
	expect(css).toContain(`color: ${expected};`);
	if (WATCH_STEP === "0") {
		STATE.previousTone = tone;
		STATE.previousComposedClass = ownClass;
	} else {
		expect(tone).not.toBe(STATE.previousTone);
		expect(ownClass).not.toBe(STATE.previousComposedClass);
		expect(css).not.toContain(`.${STATE.previousTone} {`);
		expect(css).not.toContain(`.${STATE.previousComposedClass} {`);
	}
});
