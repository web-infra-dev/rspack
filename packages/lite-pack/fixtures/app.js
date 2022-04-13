import { secret, myanswer } from './lib';

console.log('answer:',myanswer, secret);
// setTimeout(() => {
//   answer++;
// },1000)


export function render(){
  const container = document.getElementById('root');
  container.innerHTML = `adddd:${secret}:${myanswer}`
}

if(module.hot?.accept){
  module.hot.accept('./lib',(module) => {
    console.log('lib:',module);
    secret = module.secret;
    myanswer = module.myanswer;
    render();
  })
  module.hot.accept((module) => {
    console.log('xxx:', module);
    render();
  })
}