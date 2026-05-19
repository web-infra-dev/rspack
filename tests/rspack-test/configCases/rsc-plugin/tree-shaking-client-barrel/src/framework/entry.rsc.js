import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { App } from '../App';

export const renderRscStream = () => {
    return renderToReadableStream(<App />);
};

it('should keep all exports when a client boundary is upgraded to a whole-module reference', async () => {
    const { loadClientModule } = __non_webpack_require__("./static/main.js");
    const clientModule = __rspack_rsc_manifest__.clientManifest[LIB_PATH];
    const exports = await loadClientModule(clientModule.chunks[0], clientModule.id);

    expect(exports.sort()).toEqual(['Client', 'Unused']);
});
