import { reexportedBar, unusedReexport } from "./barrel";
import { bar as starBar, local } from "./star";
import { multiBar, multiFoo } from "./multi-star";
import * as shared from "./shared";

export function foo() {
	const { namespaceFoo } = shared;
	return reexportedBar() + starBar() + namespaceFoo() + local + multiFoo() + multiBar() - 57;
}

export function unusedFoo() {
	return reexportedBar();
}

export function unusedOther() {
	return unusedReexport();
}
