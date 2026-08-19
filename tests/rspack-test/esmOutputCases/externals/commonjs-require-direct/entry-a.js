const consumer = require("./consumer-a.cjs");

process.__commonjsExternalValues.push(
	consumer.first,
	consumer.second,
	consumer.named,
	consumer.viaModuleRequire,
	consumer.constructed
);

if (
	consumer.method !== 42 ||
	consumer.fromThis !== 42 ||
	consumer.destructured !== 42
) {
	throw new Error("direct CommonJS external member access should preserve semantics");
}

export const value = consumer.first.value;
