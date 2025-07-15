import { checkIsNonemptyString } from "./types";
import uuid from "./uuid";

export function UiSelectButton() {
	checkIsNonemptyString();
	uuid();
}

console.log.bind(console);
