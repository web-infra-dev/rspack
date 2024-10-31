import {
	BuiltinPluginName,
	type RawDllReferencePluginAgencyOptions
} from "@rspack/binding";
import { create } from "./base";

export type DllReferencePluginAgencyOptions =
	RawDllReferencePluginAgencyOptions;

export const DllReferenceAgencyPlugin = create(
	BuiltinPluginName.DllReferenceAgencyPlugin,
	(
		options: DllReferencePluginAgencyOptions
	): RawDllReferencePluginAgencyOptions => options
);
