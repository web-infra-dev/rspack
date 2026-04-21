import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { App } from '../App';

export const renderRscStream = () => {
	return renderToReadableStream(<App />);
};

it('should resolve css resources for client boundaries with resource queries', async () => {
	const cssFiles = __rspack_rsc_manifest__.clientManifest[CLIENT_PATH].cssFiles;
	expect(cssFiles.length).toBe(1);

	let streamOutput = '';
	const stream = await renderRscStream();
	const reader = stream.getReader();
	const decoder = new TextDecoder();
	while (true) {
		const { done, value } = await reader.read();
		if (done) break;
		if (value) streamOutput += decoder.decode(value, { stream: true });
	}
	streamOutput += decoder.decode();

	expect(streamOutput).toContain(cssFiles[0]);
});
