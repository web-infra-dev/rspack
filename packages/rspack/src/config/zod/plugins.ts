import { z } from "zod";
import { Compiler } from "../../Compiler";

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
