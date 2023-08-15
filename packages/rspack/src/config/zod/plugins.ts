import { z } from "zod";
import { Compiler } from "../../compiler";

const rspackPluginFunction = z
	.function()
	.args(z.instanceof(Compiler))
	.returns(z.void());

const rspackPluginInstance = z.object({
	name: z.string().optional(),
	apply: rspackPluginFunction
});

export function plugins() {
	const functionOrInstance = rspackPluginFunction.or(rspackPluginInstance);
	return functionOrInstance.array();
}
