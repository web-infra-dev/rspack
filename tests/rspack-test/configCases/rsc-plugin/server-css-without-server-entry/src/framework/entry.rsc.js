import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { App } from '../App';

export const renderRscStream = () => {
	return renderToReadableStream(<App />);
};

it('should not expose server-entry metadata without the directive', async () => {
	expect(App.entryJsFiles).toBeUndefined();
	expect(App.entryCssFiles).toBeUndefined();
});
