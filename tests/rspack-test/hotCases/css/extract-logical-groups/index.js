import { load, loadUnloaded } from './loader';
import './trigger';

function names() {
  return Array.from(document.getElementsByTagName('link'))
    .map(link => (link.getAttribute('href') || '').split('?')[0].split('/').pop())
    .filter(name => name.startsWith('shared-')).sort();
}

it('moves stylesheets for loaded groups without importing unloaded groups', async () => {
  const first = await load();
  if (typeof document !== 'undefined') expect(names()).toEqual(['shared-loaded-0.css']);
  for (let generation = 1; generation <= 2; generation++) {
    await NEXT_HMR();
    // The replacement stylesheet must be ready without calling import() again.
    if (typeof document !== 'undefined') expect(names()).toEqual([`shared-loaded-${generation}.css`]);
    expect(await load()).toBe(first);
  }
  await loadUnloaded();
  if (typeof document !== 'undefined') expect(names()).toEqual(['shared-loaded-2.css', 'shared-unloaded-2.css']);
});
