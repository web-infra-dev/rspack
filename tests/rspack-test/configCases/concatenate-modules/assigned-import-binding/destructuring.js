import * as counter from "./counter";
import { box, count } from "./counter";

export function writeArrayPattern() {
	[count] = [5];
}

export function writeObjectPattern() {
	({ count } = { count: 5 });
}

export function writeObjectPatternDefault() {
	({ count = 5 } = {});
}

export function writeRenamedObjectPattern() {
	({ value: count } = { value: 5 });
}

export function writeNestedPattern() {
	[{ count }] = [{ count: 5 }];
}

export function writeRestPattern() {
	[...count] = [5];
}

export function writeNamespaceMemberPattern() {
	[counter.count] = [5];
}

export function loopArrayPattern() {
	for ([count] of [[5]]) {
	}
}

export function loopObjectPattern() {
	for ({ count } of [{ count: 5 }]) {
	}
}

export function readAfterPattern() {
	try {
		[count] = [5];
	} catch (e) {}
	return count;
}

export function writeImportedObjectProperty() {
	[box.value] = [3];
	({ next: box.next } = { next: 4 });
	box.value++;
	return box.value + box.next;
}
