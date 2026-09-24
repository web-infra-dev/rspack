import { rs } from '@rstest/core';

const mockFoo = () => rs.doMock('./src/foo', () => ({ value: 'mockedFoo3' }));
const unmockFoo = () => rs.doUnmock('./src/foo');
const resetModules = () => rs.resetModules();

it('non-hoisted mock APIs should work in expression position', async () => {
	mockFoo();
	const { foo: mocked } = await import('./src/barrel');
	expect(mocked).toBe('mockedFoo3');

	unmockFoo();
	resetModules();
	const { foo: actual } = await import('./src/barrel');
	expect(actual).toBe('foo');
});
