const button = document.createElement('button');
button.textContent = 'Click me';

button.addEventListener('click', () => {
  import('./component.js').then(() => {
    console.log('Component loaded via cross-origin lazy compilation');
  });
});
document.body.appendChild(button);
