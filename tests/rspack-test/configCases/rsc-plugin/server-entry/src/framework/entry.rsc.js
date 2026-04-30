import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { App, getCssNodes, getInheritedCssNodes } from '../App';

export const renderRscStream = () => {
    return renderToReadableStream(<App />);
};

it('should expose entry JS and CSS files for server entries', async () => {
    expect(App.entryJsFiles).toBeDefined();
    expect(App.entryCssFiles).toBeDefined();

    expect(App.entryJsFiles.length).toEqual(1);
    expect(App.entryJsFiles[0]).toMatch(/\.js$/);

    expect(App.entryCssFiles.length).toEqual(1);
    expect(App.entryCssFiles[0]).toMatch(/\.css$/);
});

it('should load css from the current server entry', () => {
    const cssNodes = getCssNodes();

    expect(cssNodes.length).toEqual(1);
    expect(cssNodes[0].type).toBe('link');
    expect(cssNodes[0].props.rel).toBe('stylesheet');
    expect(cssNodes[0].props.href).toMatch(/\.css$/);
    expect(cssNodes[0].props.precedence).toBe('default');
});

it('should inherit css from the nearest parent server entry', () => {
    const cssNodes = getInheritedCssNodes();

    expect(cssNodes.length).toEqual(1);
    expect(cssNodes[0].type).toBe('link');
    expect(cssNodes[0].props.rel).toBe('stylesheet');
    expect(cssNodes[0].props.href).toMatch(/\.css$/);
    expect(cssNodes[0].props.precedence).toBe('default');
});
