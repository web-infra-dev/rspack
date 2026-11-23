import { effects } from "./effects";
import { value as calls } from "./calls";
import { value as shadowed } from "./shadowed";
import { value as math } from "./math";
import { value as argument } from "./arguments";
import { value as spread } from "./spread";
import { value as constructed } from "./constructed";
import { value as exported } from "./exported";
import { value as promise } from "./promise";
import { value as entries } from "./entries";
import { value as typed } from "./typed";
import { value as mapped } from "./mapped";
import { value as innerGraph } from "./inner-graph";

it("should treat calls to pure globals as side effect free", async () => {
	const content = await require("fs/promises").readFile(__filename, "utf-8");
	expect(calls).toBe(42);
	expect(content).not.toContain("./calls" + ".js");
});

it("should keep shadowed callees and impure arguments", async () => {
	const content = await require("fs/promises").readFile(__filename, "utf-8");
	expect([shadowed, math, argument, spread, constructed, exported]).toEqual([
		42, 42, 42, 42, 42, 42
	]);
	expect(effects.slice(0, 5)).toEqual([
		"shadowed",
		"math",
		"argument",
		"constructed",
		"exported"
	]);
	for (const module of [
		"./shadowed",
		"./math",
		"./arguments",
		"./spread",
		"./constructed",
		"./exported"
	]) {
		expect(content).toContain(module + ".js");
	}
});

it("should keep globals that run user code", async () => {
	const content = await require("fs/promises").readFile(__filename, "utf-8");
	expect([promise, entries, typed, mapped]).toEqual([42, 42, 42, 42]);
	expect(effects.slice(5, 9)).toEqual(["promise", "entries", "typed", "mapped"]);
	for (const module of ["./promise", "./entries", "./typed", "./mapped"]) {
		expect(content).toContain(module + ".js");
	}
});

it("should not let innerGraph drop unused pure global calls", () => {
	expect(innerGraph()).toBe(42);
	expect(effects.slice(9)).toEqual(["inner-graph"]);
});
