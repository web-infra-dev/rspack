import * as emptyDestructureSource from "./empty-destructure-source";

export function getEmptyDestructure() {
	const {} = emptyDestructureSource;
	return "empty-destructure";
}
