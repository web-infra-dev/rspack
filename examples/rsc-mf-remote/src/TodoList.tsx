import { getTodos } from './actions';
import { TodoItem } from './TodoItem';

export async function TodoList({ id }: { id: number | undefined }) {
  const todos = await getTodos();
  return (
    <ul className="todo-list" data-testid="todo-list">
      {todos.map((todo) => (
        <TodoItem key={todo.id} todo={todo} isSelected={todo.id === id} />
      ))}
    </ul>
  );
}
