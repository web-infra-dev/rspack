// @ts-nocheck

import { toBeTypeOf } from "./expect/to-be-typeof";
import { toEndWith } from "./expect/to-end-with";
import { toMatchFileSnapshot } from "./expect/to-match-file-snapshot";

expect.extend({
	// CHANGE: new test matcher for `rspack-test-tools`
	toMatchFileSnapshot,
	toBeTypeOf,
	toEndWith
});
