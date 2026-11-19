import { effects } from "./effects";
import { value as calls } from "./calls";
import { value as shadowed } from "./shadowed";
import { value as math } from "./math";
import { value as argument } from "./arguments";
import { value as spread } from "./spread";
import { value as constructed } from "./constructed";
import { value as exported } from "./exported";
import { value as random } from "./random";
import { value as date } from "./date";
import { value as promise } from "./promise";
import { value as entries } from "./entries";
import { value as typed } from "./typed";
import { value as mapped } from "./mapped";
import { value as declared } from "./declared";
import { value as params } from "./params";

it("should treat calls to pure globals as side effect free", async () => {
	const content = await require("fs/promises").readFile(__filename, "utf-8");
	expect(calls).toBe(42);
	expect(content).not.toContain("./calls" + ".js");
});

it("should keep shadowed callees, impure arguments and unlisted globals", async () => {
	const content = await require("fs/promises").readFile(__filename, "utf-8");
	expect([
		shadowed,
		math,
		argument,
		spread,
		constructed,
		exported,
		random,
		date
	]).toEqual([42, 42, 42, 42, 42, 42, 42, 42]);
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
		"./exported",
		"./random",
		"./date"
	]) {
		expect(content).toContain(module + ".js");
	}
});

it("should keep globals that run user code", async () => {
	const content = await require("fs/promises").readFile(__filename, "utf-8");
	expect([promise, entries, typed, mapped, declared, params]).toEqual([
		42, 42, 42, 42, 42, 42
	]);
	expect(effects.slice(5)).toEqual([
		"promise",
		"entries",
		"typed",
		"mapped",
		"declared",
		"params"
	]);
	for (const module of [
		"./promise",
		"./entries",
		"./typed",
		"./mapped",
		"./declared",
		"./params"
	]) {
		expect(content).toContain(module + ".js");
	}
});
