import { answer } from 'file://../src/answer.js';
function render() {
  document.getElementById(
    'root'
  ).innerHTML = `the answer to the universe is ${answer}`;
}
render();