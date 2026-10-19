import { create } from './lib/index.js';
import local from './default-barrel.js';

sessionStorage.setItem(
  'documentLoads',
  String(Number(sessionStorage.getItem('documentLoads')) + 1),
);
document.body.dataset.infra = create('initial').state;
document.body.dataset.default = local();

const button = document.createElement('button');
button.textContent = 'Activate feature';
button.onclick = async () => {
  const { value } = await import('./feature.js');
  document.body.dataset.feature = value;
};
document.body.appendChild(button);
