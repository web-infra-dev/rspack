import * as counter from "./counter";
import { count } from "./counter";

export function loopOfIdentifier() {
	for (count of [7]) {
	}
}

export function loopInIdentifier() {
	for (count in { seven: 7 }) {
	}
}

export function loopOfNamespaceMember() {
	for (counter.count of [8]) {
	}
}

export async function loopAwaitIdentifier() {
	for await (count of [Promise.resolve(9)]) {
	}
}
