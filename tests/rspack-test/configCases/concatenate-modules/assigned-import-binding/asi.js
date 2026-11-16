import * as counter from "./counter"
import { count } from "./counter"
import { reexportedDefault } from "./reexport-cjs"

function noop() {}

export function asiAssign() {
	noop()
	count = 1
}

export function asiUpdate() {
	noop()
	counter.count++
}

export function asiAssignReexportedDefault() {
	noop()
	reexportedDefault = 2
}

export function asiKeepTruthyReexportedDefault() {
	noop()
	reexportedDefault ||= 2
	return reexportedDefault
}
