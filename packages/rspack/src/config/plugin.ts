import { Compiler } from "..";

export interface PluginInstance {
	name?: string;
	apply(compiler: Compiler): void;
}
