const runButton = document.querySelector('#run');
const result = document.querySelector('#result');

runButton.addEventListener('click', () => {
  result.textContent = 'Action executed.';
});
