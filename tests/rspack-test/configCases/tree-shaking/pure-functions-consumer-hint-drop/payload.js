// A side-effects-free helper used as the nested argument. It is reached only by
// the unused `identity(payload())` call, so correct resolution drops it too.
export function payload() {
	return "pure-functions-consumer-hint::payload-sentinel";
}
