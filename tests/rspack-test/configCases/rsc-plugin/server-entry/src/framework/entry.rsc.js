import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { App } from '../App';

export const renderRscStream = () => {
    return renderToReadableStream(<App />);
};

it('should preserve all server actions in production build', async () => {
    expect(App.entryJsFiles).toBeDefined();
    expect(App.entryCssFiles).toBeDefined();

    expect(App.entryJsFiles.length).toEqual(1);
    expect(App.entryJsFiles[0]).toMatch(/\.js$/);

    expect(App.entryCssFiles.length).toEqual(1);
    expect(App.entryCssFiles[0]).toMatch(/\.css$/);
});
