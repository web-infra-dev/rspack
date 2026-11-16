import * as counter from "./counter";

export function reassignNamespace() {
	counter = 1;
}

export function updateNamespace() {
	counter++;
}

export function loopNamespace() {
	for (counter of [1]) {
	}
}

export function namespaceObject() {
	return counter;
}
