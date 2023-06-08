import { z } from "zod";

const externalItem = z
	.instanceof(RegExp)
	.or(z.string())
	.or(z.function())
	.or(z.any());

export function externals() {
	return externalItem.or(externalItem.array());
}
