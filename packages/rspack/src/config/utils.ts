import { z } from "zod/v4";

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
