import './local.css';

const links = () => Array.from(document.querySelectorAll('link[rel="stylesheet"]'));
const names = () => links().map(link => new URL(link.href).pathname.split('/').pop().split('.')[0]).sort();

it('loads CSS split from a dependOn ancestor', async () => {
  expect(names()).toEqual(['base', 'main']);
  const local = links()[1];
  await NEXT_HMR();
  expect(names()).toEqual(['main', 'shared']);
  expect(links()).toContain(local);
});
