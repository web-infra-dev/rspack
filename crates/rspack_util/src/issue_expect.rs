pub trait IssueExpect<T> {
  fn issue_expect(self) -> T;
}

impl<T, E: std::fmt::Debug> IssueExpect<T> for Result<T, E> {
  fn issue_expect(self) -> T {
    self.expect("This should never happen, please file an issue")
  }
}

impl<T> IssueExpect<T> for Option<T> {
  fn issue_expect(self) -> T {
    self.expect("This should never happen, please file an issue")
  }
}
