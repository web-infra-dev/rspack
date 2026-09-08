import { createRequire } from 'node:module';

const resolve = require.resolve(/* webpackIgnore: true */ "node:fs");
const createRequireResolve1 = createRequire(import.meta.url).resolve(/* webpackIgnore: true */ "node:fs");
const createdRequire = createRequire(import.meta.url);
const createRequireResolve2 = createdRequire.resolve(/* webpackIgnore: true */ "node:fs");
const rspackResolve = createdRequire.resolve(/* rspackIgnore: true */ "node:fs");
const rspackCreateRequireResolve1 = createRequire(import.meta.url).resolve(/* rspackIgnore: true */ "node:fs");
const rspackCreateRequireResolve2 = createdRequire.resolve(/* rspackIgnore: true */ "node:fs");

const ignoredMissing = () => [
	require.resolve(/* webpackIgnore: true */ "./non-exists"),
	require.resolve(/* rspackIgnore: true */ "./non-exists")
];

export {
	resolve,
	createRequireResolve1,
	createRequireResolve2,
	rspackResolve,
	rspackCreateRequireResolve1,
	rspackCreateRequireResolve2,
	ignoredMissing,
}
