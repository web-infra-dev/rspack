import {
	loadServerAction,
	renderToReadableStream
} from "react-server-dom-rspack/server";
import { App } from "../App";

const RSC_ACTION_ERROR =
	'A "use server" file can only export async functions, found number.';

export const renderRscStream = () => {
	return renderToReadableStream(<App />);
};

const getActionIds = () => {
	const manifest = __rspack_rsc_manifest__;
	expect(manifest).toBeDefined();
	expect(manifest.serverManifest).toBeDefined();

	return Object.keys(manifest.serverManifest);
};

it("should reject non-function server action exports with rscA at runtime", () => {
	const actionIds = getActionIds();

	expect(actionIds).toHaveLength(2);
	expect(() => {
		for (const actionId of actionIds) {
			loadServerAction(actionId);
		}
	}).toThrow(RSC_ACTION_ERROR);
});
