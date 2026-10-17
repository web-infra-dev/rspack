import { infra } from './infrastructure.js';

sessionStorage.setItem(
  'documentLoads',
  String(Number(sessionStorage.getItem('documentLoads')) + 1),
);
document.body.dataset.infra = infra;

const button = document.createElement('button');
button.textContent = 'Activate feature';
button.onclick = async () => {
  const { value } = await import('./feature.js');
  document.body.dataset.feature = value;
};
document.body.appendChild(button);
