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
  module.hot.accept('/Users/yangjian/github/rspack/packages/lite-pack/fixtures/lib.js',(module) => {
    console.log('lib:',module);
    render();
  })
  module.hot.accept('/Users/yangjian/github/rspack/packages/lite-pack/fixtures/answer.js',(module) => {
    console.log('answer:',module);
    render();
  })
  module.hot.accept((module) => {
    console.log('xxx:', module);
    render();
  })
}