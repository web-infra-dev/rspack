import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { App } from '../App';

export const renderRscStream = () => {
    return renderToReadableStream(<App />);
};

it('should tree-shake unused exports from "use client" modules', async () => {
    const { loadClientModule } = require("./static/main.js");
    const chunkId = __rspack_rsc_manifest__.clientManifest[TODOS_PATH].chunks[0];
    const moduleId = __rspack_rsc_manifest__.clientManifest[TODOS_PATH].id;
    const exports = await loadClientModule(chunkId, moduleId);
    expect(exports.sort()).toEqual(['TodoItem', 'Todos']);
});

it('should preserve the exports object for dynamically imported client modules', async () => {
    const { loadClientModule } = require("./static/main.js");
    const dynamicChunkId = __rspack_rsc_manifest__.clientManifest[DYNAMIC_TODO_PATH].chunks[0];
    const dynamicModuleId = __rspack_rsc_manifest__.clientManifest[DYNAMIC_TODO_PATH].id;
    const dynamicExports = await loadClientModule(dynamicChunkId, dynamicModuleId);
    expect(dynamicExports.sort()).toEqual(['DynamicTodo', 'Unused']);
});
