const context = require.context('./', false, /mod/);

console.log(context);

document.getElementById('root').textContent = '__PAGE_RENDER__';

if (import.meta.webpackHot) {
  import.meta.webpackHot.accept();
  import.meta.webpackHot.addStatusHandler((status) => {
    if (status === 'idle') {
      document.getElementById('root').textContent = '__HMR_UPDATED__';
    }
  });
}
