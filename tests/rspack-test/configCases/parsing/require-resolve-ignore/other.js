import { createRequire } from 'node:module';

const resolve = require.resolve(/* webpackIgnore: true */ "./non-exists");
const createRequireResolve1 = createRequire(import.meta.url).resolve(/* webpackIgnore: true */ "./non-exists");
const require = createRequire(import.meta.url);
const createRequireResolve2 = require.resolve(/* webpackIgnore: true */ "./non-exists");
const rspackResolve = require.resolve(/* rspackIgnore: true */ "./non-exists");
const rspackCreateRequireResolve1 = createRequire(import.meta.url).resolve(/* rspackIgnore: true */ "./non-exists");
const rspackCreateRequireResolve2 = require.resolve(/* rspackIgnore: true */ "./non-exists");

export {
	resolve,
	createRequireResolve1,
	createRequireResolve2,
	rspackResolve,
	rspackCreateRequireResolve1,
	rspackCreateRequireResolve2,
}
