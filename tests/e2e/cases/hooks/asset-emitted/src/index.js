import('./foo').then((m) => {
  document.getElementById('root').innerText = `__ROOT_TEXT__${m.foo}`;
});
