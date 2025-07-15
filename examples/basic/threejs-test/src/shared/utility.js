export class SharedUtility {
  process() {
    return 'utility-processed';
  }
  
  transform(data) {
    return data.map(x => x * 2);
  }
}
