import { evaluations } from "./state";
import { getBasename } from "./lib";

evaluations.push("consumer");

export function getName() {
	return getBasename("/example/file.txt");
}
