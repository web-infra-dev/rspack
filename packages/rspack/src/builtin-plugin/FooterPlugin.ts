import { RawFooterPluginOptions } from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";
export const FooterRspackPlugin = create(
	BuiltinPluginName.FooterRspackPlugin,
	(args: { footer: string }): RawFooterPluginOptions => {
		return {
			footer: args.footer
		};
	}
);
