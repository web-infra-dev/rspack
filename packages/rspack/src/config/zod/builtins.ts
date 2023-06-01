import { z } from "zod";

export function builtins() {
	// TODO(hyf0): need to enable strict mode sometimes
	return z.record(z.any());
}
