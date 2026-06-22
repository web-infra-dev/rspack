import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { PageA } from '../pages/PageA';
import { PageANested } from '../pages/PageANested';
import { PageB } from '../pages/PageB';
import { Root } from '../Root';

export const renderRscStream = () => {
	return renderToReadableStream(<Root />);
};

it('should expose inlined entry CSS for server-entry components', async () => {
	expect(Root.entryCssFiles).toBeUndefined();

	expect(PageA.entryCssFiles).toBeDefined();
	expect(PageA.entryCssFiles.length).toBe(1);
	expect(PageA.entryCssFiles[0]).toMatch(/\.css$/);

	expect(PageANested.entryCssFiles).toBeDefined();
	expect(PageANested.entryCssFiles.length).toBe(1);
	expect(PageANested.entryCssFiles[0]).toMatch(/\.css$/);

	expect(PageB.entryCssFiles).toBeDefined();
	expect(PageB.entryCssFiles.length).toBe(1);
	expect(PageB.entryCssFiles[0]).toMatch(/\.css$/);

	expect(PageA.entryCssFiles[0]).toBe(PageANested.entryCssFiles[0]);
	expect(PageA.entryCssFiles[0]).toBe(PageB.entryCssFiles[0]);
});
