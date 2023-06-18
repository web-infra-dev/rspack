import { z } from "zod";

// Some schemas are too complex to implement them at once.
// We use this function to get the correct type definition but ignore the validating.
export function mock<T>() {
	return z.custom<T>(() => true);
}
