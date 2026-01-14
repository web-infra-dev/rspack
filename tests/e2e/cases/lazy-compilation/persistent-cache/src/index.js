const button = document.createElement('button');
button.textContent = 'Click me';
button.id = 'click_button';
document.body.appendChild(button);

button.addEventListener('click', async () => {
  import('./dyn.js');
});
