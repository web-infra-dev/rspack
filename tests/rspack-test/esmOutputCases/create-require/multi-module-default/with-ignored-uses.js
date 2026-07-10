import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

const ignoredRequired = require(/* webpackIgnore: true */ "path");
export const ignoredRequiredJoinType = typeof ignoredRequired.join;
export const ignoredResolved = require.resolve(
	/* webpackIgnore: true */ "path"
);
export const inlineIgnoredResolved = createRequire(import.meta.url).resolve(
	/* webpackIgnore: true */ "path"
);
const inlineIgnoredRequired = createRequire(import.meta.url)(
	/* webpackIgnore: true */ "path"
);
export const inlineIgnoredRequiredJoinType = typeof inlineIgnoredRequired.join;
