import { ECompilerType } from "../type";
import { BasicRunner } from "./basic";

export class WebRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends BasicRunner<T> {}
