import { nestedObj, starNestedLocal } from "./star-nested-barrel";

export function getStarNestedUsed() {
	return nestedObj.used + starNestedLocal;
}
