import { z } from "zod";

export function builtins() {
	// TODO(hyf0): need to enable strict mode when developer have time to write these schema
	return z.record(z.any());
}
