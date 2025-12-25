import('./async').then(() => {
  const div = document.createElement('div');
  div.innerText = 'ok';
  div.id = 'status';
  document.body.appendChild(div);
});
