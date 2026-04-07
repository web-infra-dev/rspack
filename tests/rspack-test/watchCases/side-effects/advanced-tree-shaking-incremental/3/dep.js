import { tracker } from "./tracker";

export function check() {
	tracker.push("impure-but-unused");
}
