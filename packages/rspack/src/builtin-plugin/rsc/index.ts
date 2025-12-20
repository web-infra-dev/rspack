import { RspackPluginInstance } from "../..";
import { Coordinator } from "./coordinator";
import { RscClientPlugin } from "./RscClientPlugin";
import { RscServerPlugin } from "./RscServerPlugin";

export function createRscPlugins(): {
	ServerPlugin: RspackPluginInstance;
	ClientPlugin: RspackPluginInstance;
} {
	const coordinator = new Coordinator();
	return {
		ServerPlugin: class ServerPlugin extends RscServerPlugin {
			constructor() {
				super(coordinator);
			}
		},
		ClientPlugin: class ClientPlugin extends RscClientPlugin {
			constructor() {
				super(coordinator);
			}
		}
	};
}
