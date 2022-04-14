
async function render(){
  const { answer } = await import('./answer');
  console.log('answer:',answer);
  document.getElementById('root').innerHTML = answer;
}

render();
