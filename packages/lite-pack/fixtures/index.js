import { secret, myanswer } from './lib';
console.log('answer:',myanswer, secret);
// setTimeout(() => {
//   answer++;
// },1000)
function render(){
  const container = document.getElementById('root');
  container.innerHTML = `answer66:${secret}`
}

render();

if(module.hot?.accept){
  module.hot.accept((module) => {
    console.log('xxx:', module);
    render();
  })
  module.hot.accept('./lib',(module) => {
    console.log('lib:',module)
  })
}