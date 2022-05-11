export default function checkLogin() {
  return localStorage.getItem('userStatus') === 'login';
}
