import { tripled } from './shared-three';

it('should extract the module with the larger total size reduction', () => {
	expect(tripled).toBe(42);
});
