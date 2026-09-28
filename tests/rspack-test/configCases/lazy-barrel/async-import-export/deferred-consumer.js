import { evaluations } from "./state";
import { value } from "./lib/deferred-barrel";

evaluations.push("consumer");

export function getName() {
	return value;
}
