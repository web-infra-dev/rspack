import fs from "fs";
import addDefault, { value as aliased } from "./lib";
import * as namespace from "./lib";
import { fromHelper } from "./helper";

const base64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

function decodeVlq(digits) {
	const values = [];
	let value = 0;
	let factor = 1;
	for (const char of digits) {
		const digit = base64.indexOf(char);
		expect(digit).toBeGreaterThanOrEqual(0);
		value += (digit & 31) * factor;
		if (digit & 32) {
			factor *= 32;
		} else {
			values.push(value);
			value = 0;
			factor = 1;
		}
	}
	expect(factor).toBe(1);
	return values;
}

function signed(value) {
	return value % 2 === 0 ? value / 2 : -(value >> 1);
}

function decodeBindings(map) {
	const bindings = new Map();
	for (const ranges of map.ranges) {
		let source = 0;
		for (const item of ranges.split(",")) {
			if (item[0] === "E") {
				const [flags, ...fields] = decodeVlq(item.slice(1));
				expect(flags & 2).toBe(2);
				source += signed(fields[flags & 1 ? 2 : 1]);
				expect(fields.at(-1)).toBe(0);
			}
			if (item[0] === "G") {
				const items = map.scopes[source].split(",");
				expect(items[0]).toMatch(/^BCAA/);
				expect(map.names[signed(decodeVlq(items[0].slice(4))[0])]).toBe("Module");
				expect(items[1][0]).toBe("D");
				expect(items[2][0]).toBe("C");
				let name = 0;
				const variables = decodeVlq(items[1].slice(1)).map(delta => {
					name += signed(delta);
					return map.names[name];
				});
				const expressions = decodeVlq(item.slice(1)).map(index => map.names[index - 1]);
				expect(expressions).toHaveLength(variables.length);
				bindings.set(map.sources[source], Object.fromEntries(
					variables.map((variable, i) => [variable, expressions[i]])
				));
			}
		}
	}
	return bindings;
}

it("maps original ESM import names to their generated bindings", () => {
	expect(addDefault(aliased)).toBe(42);
	expect(namespace.value).toBe(21);
	expect(fromHelper).toBe(21);

	const bundle = fs.readFileSync(__filename, "utf8");
	const maps = CONFIG_INDEX === 1 || CONFIG_INDEX === 2
		? [...bundle.matchAll(/sourceMappingURL=data:application\/json;charset=utf-8;base64,([\w+/=]+)/g)]
			.map(match => JSON.parse(Buffer.from(match[1], "base64").toString("utf8")))
		: [JSON.parse(fs.readFileSync(__filename + ".map", "utf8"))];
	expect(maps.length).toBeGreaterThan(0);

	const bindings = new Map();
	for (const map of maps) {
		if (CONFIG_INDEX === 1) expect(map.sourcesContent).toBeUndefined();
		if (!map.scopes) {
			expect(map.ranges).toBeUndefined();
			continue;
		}
		expect(map.scopes).toHaveLength(map.sources.length);
		for (const [source, values] of decodeBindings(map)) bindings.set(source, values);
	}
	if (CONFIG_INDEX === 4) {
		expect(bindings.size).toBe(0);
		return;
	}
	const entry = [...bindings].find(([source]) => source.split("?")[0].endsWith("/index.js"))?.[1];
	const helper = [...bindings].find(([source]) => source.split("?")[0].endsWith("/helper.js"))?.[1];
	expect(entry).toBeDefined();
	expect(entry.namespace).toMatch(/^[\w$]+$/);
	expect(entry.aliased).toBe(`${entry.namespace}.value`);
	expect(entry.addDefault).toBe(`${entry.namespace}["default"]`);
	expect(helper?.fromLib).toMatch(/\.value$/);
	const code = CONFIG_INDEX === 2 ? bundle.replaceAll('\\"', '"') : bundle;
	for (const expression of Object.values(entry)) {
		expect(code).toContain(expression);
	}
});
