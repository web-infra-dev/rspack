import { checkIsNonemptyString } from "./types";
import uuid from "./uuid";

export { UiSelectButton } from "./select";
export { UiSelectButton2 } from "./select2";

export function UiButton() {
	checkIsNonemptyString();
	uuid();
}
