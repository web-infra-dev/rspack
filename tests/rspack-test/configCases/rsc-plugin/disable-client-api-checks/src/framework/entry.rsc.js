import {
    loadServerAction,
    renderToReadableStream,
} from 'react-server-dom-rspack/server.node';
import { App } from '../App';

export const renderRscStream = () => {
    return renderToReadableStream(<App />);
};

it('should build successfully with disableClientApiChecks even though useState is used in a server component', async () => {
    // The build itself succeeding is the main assertion — without
    // `disableClientApiChecks: true`, the SWC RSC transform would emit a
    // compile-time error for importing `useState` in a server component.

    // At runtime, calling useState in a server context will throw because
    // React server runtime does not provide client-only hooks.
    // React's internal work scheduler handles the error asynchronously,
    // so we capture it via the `onError` callback.
    const consoleErrorSpy = rstest.spyOn(console, 'error').mockImplementation(() => { });

    let streamOutput = '';
    const stream = renderToReadableStream(<App />, null);
    const resolvedStream = await Promise.resolve(stream);

    const reader = resolvedStream.getReader();
    const decoder = new TextDecoder();
    while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        if (value) streamOutput += decoder.decode(value, { stream: true });
    }
    streamOutput += decoder.decode();

    expect(streamOutput).toMatch(/E\{"digest"/);

    const consoleOutput = consoleErrorSpy.mock.calls.flat().join(' ');
    expect(consoleOutput).toMatch(/useState/);

    consoleErrorSpy.mockRestore();
});
