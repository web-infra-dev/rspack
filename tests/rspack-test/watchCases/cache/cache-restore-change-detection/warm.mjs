import assert from "node:assert/strict";
import { promisify } from "node:util";
import rspack from "@rspack/core";

const compiler = rspack(JSON.parse(process.argv[2]));

try {
	const stats = await promisify(compiler.run.bind(compiler))();
	assert.ok(stats, "Warm build returned no stats");
	assert.equal(
		stats.hasErrors(),
		false,
		stats.toString({ all: false, errors: true, errorDetails: true })
	);
} finally {
	// Flush the filesystem cache before the watch process tries to restore it.
	await promisify(compiler.close.bind(compiler))();
}
