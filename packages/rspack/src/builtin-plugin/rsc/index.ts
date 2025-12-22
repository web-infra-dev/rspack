import { RspackPluginInstance } from "../..";
import { Coordinator } from "./Coordinator";
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

export const RSC_LAYERS_NAMES = {
	/**
	 * The layer for server-only runtime and picking up `react-server` export conditions.
	 */
	reactServerComponents: "react-server-components",
	/**
	 * Server Side Rendering layer for app.
	 */
	serverSideRendering: "server-side-rendering",
	/**
	 * The browser client bundle layer for actions.
	 */
	actionBrowser: "action-browser"
};
