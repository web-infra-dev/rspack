import small from './asset.txt?small';
import large from './asset.txt?large';
import automatic from './asset.txt?auto';
import resource from './asset.txt?resource';

it('distinguishes inline data from separately emitted assets', () => {
  expect(small).toMatch(/^data:text\/plain/);
  expect(large).toMatch(/^data:text\/plain/);
  expect(automatic).toMatch(/^data:text\/plain/);
  expect(resource).not.toMatch(/^data:/);
});
