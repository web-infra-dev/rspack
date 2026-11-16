import { count, value as renamed } from "./counter";

export function orAssign() {
	count ||= 5;
}

export function andAssign() {
	renamed &&= 5;
}

export function writeRenamed() {
	renamed = 5;
}

export function updateRenamed() {
	renamed++;
}
