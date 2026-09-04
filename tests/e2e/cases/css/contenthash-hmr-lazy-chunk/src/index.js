document.getElementById('load').addEventListener('click', () => {
  import('./lazy.js');
});

module.hot.accept();
