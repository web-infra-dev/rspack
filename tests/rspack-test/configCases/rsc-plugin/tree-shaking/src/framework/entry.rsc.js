import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { App } from '../App';

export const renderRscStream = () => {
    return renderToReadableStream(<App />);
};

it('should tree-shake unused exports from "use client" modules', async () => {
    const { loadClientModule } = __non_webpack_require__("./static/main.js");
    const chunkId = __rspack_rsc_manifest__.clientManifest[TODOS_PATH].chunks[0];
    const moduleId = __rspack_rsc_manifest__.clientManifest[TODOS_PATH].id;
    const exports = await loadClientModule(chunkId, moduleId);
    expect(exports).toEqual(['Todos']);
});
