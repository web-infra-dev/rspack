import { BuiltinPluginName, RawIgnorePluginOptions } from "@rspack/binding";
import { create } from "./base";

export type IgnorePluginOptions = RawIgnorePluginOptions;

export const IgnorePlugin = create(
	BuiltinPluginName.IgnorePlugin,
	(options: IgnorePluginOptions): RawIgnorePluginOptions => options
);
