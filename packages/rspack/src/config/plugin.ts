import { Compiler } from "..";

export interface Plugin {
	name: string;
	apply(compiler: Compiler): void;
}
