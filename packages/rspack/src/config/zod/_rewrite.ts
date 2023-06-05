import { z } from "zod";
import { configSchema } from "./index";
import type {
	Output,
	Resolve,
	ModuleOptions,
	Plugins,
	Builtins,
	DevServer
} from "../types";

// The final goal is to infer the type using the schema without any rewriting.
// But currently there are some schema are loose, so we need to rewrite the `Config`
// type to expose the correct type to users.
type Config = z.infer<ReturnType<typeof configSchema>>;

type Rewritten = {
	output?: Output;
	resolve?: Resolve;
	module?: ModuleOptions;
	plugins?: Plugins;
	builtins?: Builtins;
	devServer?: DevServer;
};

export type Options = Omit<Config, keyof Rewritten> & Rewritten;
