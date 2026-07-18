import { doubled } from './shared-two';
import { tripled } from './shared-three';

it('should extract the module with the larger total size reduction', () => {
	expect(doubled + tripled).toBe(63);
});
