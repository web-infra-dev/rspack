import { Rspack } from "..";

export interface Plugin {
	name: string;
	apply(compiler: Rspack): void;
}
