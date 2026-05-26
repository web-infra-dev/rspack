import * as styles from "./entry.module.css";
import * as cycleStyles from "./cycle-entry.module.css";

it("should keep composed css modules in cascade order", () => {
	expect(styles.one).toBe("one b c");
	expect(styles.two).toBe("two a");

	const fs = __non_webpack_require__("fs");
	const cssFile = fs.readdirSync(__dirname).find(file => file.endsWith(".css"));
	const css = fs.readFileSync(`${__dirname}/${cssFile}`, "utf-8");
	const reset = css.indexOf(".reset");
	const a = css.indexOf(".a");
	const b = css.indexOf(".b");
	const c = css.indexOf(".c");

	expect(reset).toBeGreaterThanOrEqual(0);
	expect(a).toBeGreaterThanOrEqual(0);
	expect(reset).toBeLessThan(b);
	expect(b).toBeGreaterThanOrEqual(0);
	expect(c).toBeGreaterThan(b);
	expect(a).toBeGreaterThan(c);
});

it("should keep cyclic composed css modules in source order", () => {
	expect(cycleStyles.seen).toBe("seen q");
	expect(cycleStyles.cycleOne).toBe("cycleOne x y");
	expect(cycleStyles.cycleTwo).toBe("cycleTwo y x");
	expect(cycleStyles.later).toBe("later p q");
	expect(cycleStyles.independent).toBe("independent z");

	const fs = __non_webpack_require__("fs");
	const cssFile = fs.readdirSync(__dirname).find(file => file.endsWith(".css"));
	const css = fs.readFileSync(`${__dirname}/${cssFile}`, "utf-8");
	const p = css.indexOf(".p");
	const q = css.indexOf(".q");
	const x = css.indexOf(".x");
	const y = css.indexOf(".y");
	const z = css.indexOf(".z");

	expect(p).toBeGreaterThanOrEqual(0);
	expect(p).toBeLessThan(q);
	expect(x).toBeGreaterThan(p);
	expect(x).toBeGreaterThanOrEqual(0);
	expect(y).toBeGreaterThan(x);
	expect(q).toBeGreaterThan(y);
	expect(z).toBeGreaterThan(q);
});
