import {
    loadServerAction,
    renderToReadableStream,
} from 'react-server-dom-rspack/server';
import { App } from '../App';

export const renderRscStream = () => {
    return renderToReadableStream(<App />);
};

it('should preserve all server actions in production build', async () => {
    const manifest = __rspack_rsc_manifest__;
    expect(manifest).toBeDefined();

    const { serverManifest } = manifest;
    expect(serverManifest).toBeDefined();

    const actionIds = Object.keys(serverManifest);
    expect(actionIds).toHaveLength(4);

    // Ensure all collected actions are loadable server actions.
    actionIds.forEach((actionId) => {
        expect(loadServerAction(actionId)).toEqual(expect.any(Function));
    });
});
