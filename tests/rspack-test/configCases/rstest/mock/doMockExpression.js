import { rs } from '@rstest/core';

const mockFoo = () => rs.doMock('./src/foo', () => ({ value: 'mockedFoo3' }));
const unmockFoo = () => rs.doUnmock('./src/foo');
const resetModules = () => rs.resetModules();

// The sibling `require` must still be collected as a dependency.
const mockBar = rs.doMock('./src/bar', () => ({ value: 'mockedBar' })),
	baz = require('./src/baz');

it('non-hoisted mock APIs should work in expression position', async () => {
	mockFoo();
	const { foo: mocked, bar } = await import('./src/barrel');
	expect(mocked).toBe('mockedFoo3');
	expect(bar).toBe('mockedBar');
	expect(baz.value).toBe('baz');

	unmockFoo();
	resetModules();
	const { foo: actual } = await import('./src/barrel');
	expect(actual).toBe('foo');
});
