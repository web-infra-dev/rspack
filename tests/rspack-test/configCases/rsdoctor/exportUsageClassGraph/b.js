import { baz, unusedBaz } from "./c";

export function bar() {
	return baz();
}

export function unusedBar() {
	return unusedBaz();
}
