import { answer } from './answer';
console.log('answer:',answer);
setTimeout(() => {
  answer++;
},1000)
function render(){
  const container = document.getElementById('root');
  container.innerHTML = `answer88:${answer}`
}
render();

if(module.hot?.accept){
  module.hot.accept((module) => {

    console.log('xxx:', module);
    render();
  })
}