import { createTodo } from './actions';

export function TodoCreate() {
  return (
    <form data-testid="todo-create-form" action={createTodo}>
      <label>
        Title: <input name="title" />
      </label>
      <label>
        Description: <textarea name="description" />
      </label>
      <label>
        Due date: <input type="date" name="dueDate" />
      </label>
      <button type="submit" data-testid="todo-create-submit">
        Add todo
      </button>
    </form>
  );
}
