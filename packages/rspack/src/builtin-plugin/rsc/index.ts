import { RspackPluginInstance } from "../..";
import { Coordinator } from "./coordinator";
import { ReactClientPlugin } from "./ReactClientPlugin";
import { ReactServerPlugin } from "./ReactServerPlugin";

export function createRscPlugins(): {
	ServerPlugin: RspackPluginInstance;
	ClientPlugin: RspackPluginInstance;
} {
	const coordinator = new Coordinator();
	return {
		ServerPlugin: class ServerPlugin extends ReactServerPlugin {
			constructor() {
				super(coordinator);
			}
		},
		ClientPlugin: class ClientPlugin extends ReactClientPlugin {
			constructor() {
				super(coordinator);
			}
		}
	};
}
