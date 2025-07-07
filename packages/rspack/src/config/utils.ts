import { z } from "./zod";

// Zod v4 doesn't support Infinity, so we need to use a custom type
// See: https://github.com/colinhacks/zod/issues/4721
export const numberOrInfinity = z
	.number()
	.or(z.literal(Number.POSITIVE_INFINITY));

export const anyFunction = z.custom<(...args: unknown[]) => any>(
	data => typeof data === "function",
	// Make the similar error message as zod v3
	// https://github.com/colinhacks/zod/blob/64bfb7001cf6f2575bf38b5e6130bc73b4b0e371/packages/zod/src/v3/types.ts#L3821-L3828
	{
		error: input => ({
			message: `Expected function, received ${z.core.util.getParsedType(input)}`
		})
	}
);
