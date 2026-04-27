import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { Page1 } from '../pages/Page1';
import { Page2 } from '../pages/Page2';
import { Root } from '../Root';

export const renderRscStream = () => {
	return renderToReadableStream(<Root />);
};

it('should expose entry CSS only for server-entry components', async () => {
	expect(Root.entryCssFiles).toBeUndefined();

	expect(Page1.entryCssFiles).toBeDefined();
	expect(Page1.entryCssFiles.length).toBe(1);
	expect(Page1.entryCssFiles[0]).toMatch(/\.css$/);

	expect(Page2.entryCssFiles).toBeDefined();
	expect(Page2.entryCssFiles.length).toBe(1);
	expect(Page2.entryCssFiles[0]).toMatch(/\.css$/);

	expect(Page1.entryCssFiles[0]).not.toBe(Page2.entryCssFiles[0]);
});
