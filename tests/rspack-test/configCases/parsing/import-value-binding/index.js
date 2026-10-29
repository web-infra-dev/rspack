import fs from "node:fs";
import path from "node:path";
import config from "./cjs-settings";
import { merge, identity } from "./utils";
import liveDefault, { setCurrent } from "./live-default";
import { count, bump } from "./mutated-named";
import { value as evalValue } from "./eval-target";
import cycleA from "./cycle-a";

it("should snapshot CJS default interop once per importer", () => {
	expect(config.enabled).toBe(true);
	expect(config.label).toBe("cjs-settings");
	const source = fs.readFileSync(path.resolve(__dirname, "bundle0.js"), "utf-8");
	expect(source).toMatch(/_default_value\s*=\s*\w+_default\(\)/);
	expect(source).toMatch(/_default_value\.enabled/);
});

it("should still resolve unmutated named imports", () => {
	expect(merge({ a: 1 }, { b: 2 })).toEqual({ a: 1, b: 2 });
	expect(identity(3)).toBe(3);
});

it("should keep live default bindings mutable", () => {
	expect(liveDefault).toBe("initial");
	setCurrent("changed");
	expect(liveDefault).toBe("changed");
});

it("should keep mutated named exports live", () => {
	expect(count).toBe(0);
	bump();
	expect(count).toBe(1);
});

it("should keep imports from modules with direct eval working", () => {
	expect(evalValue).toBe("eval-target");
});

it("should keep circular default imports working", () => {
	expect(typeof cycleA).toBe("function");
	expect(typeof cycleA()).toBe("function");
	expect(cycleA()()).toBe("function");
});
