import { answer } from './answer';
console.log('answer:',answer);

function render(){
  const container = document.getElementById('root');
  container.innerHTML = `answer:${answer}`
}
render();
setInterval(() => {
  answer++;
  render();
},1000)