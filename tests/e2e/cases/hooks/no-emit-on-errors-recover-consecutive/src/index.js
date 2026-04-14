const div = document.createElement('div');
div.id = 'root';
div.innerText = 'value:1';
document.body.appendChild(div);

if (module.hot) {
  module.hot.accept();
}
