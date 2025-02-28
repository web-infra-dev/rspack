import {
	BuiltinPluginName,
	type RawFlightClientEntryPluginOptions
} from "@rspack/binding";

import { create } from "./base";

export type FlightClientEntryPluginOptions = RawFlightClientEntryPluginOptions;

export const FlightClientEntryPlugin = create(
	BuiltinPluginName.FlightClientEntryPlugin,
	(options: FlightClientEntryPluginOptions) => options
);
