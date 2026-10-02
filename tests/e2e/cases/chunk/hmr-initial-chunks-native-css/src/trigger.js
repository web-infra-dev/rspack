document.querySelector('#load').onclick = () => {
  import('./feature.js').then(() => {
    document.querySelector('#root').textContent = 'loaded';
  });
};
