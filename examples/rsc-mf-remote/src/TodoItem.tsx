'use client';

import { startTransition, useOptimistic } from 'react';
import { deleteTodo, type Todo as ITodo, setTodoComplete } from './actions';

export function TodoItem({
  todo,
  isSelected,
}: {
  todo: ITodo;
  isSelected: boolean;
}) {
  const [isOptimisticComplete, setOptimisticComplete] = useOptimistic(
    todo.isComplete,
  );

  return (
    <li
      data-selected={isSelected || undefined}
      data-testid={`todo-item-${todo.id}`}
    >
      <input
        data-testid={`todo-item-checkbox-${todo.id}`}
        type="checkbox"
        checked={isOptimisticComplete}
        onChange={(e) => {
          startTransition(async () => {
            setOptimisticComplete(e.target.checked);
            await setTodoComplete(todo.id, e.target.checked);
          });
        }}
      />
      <a
        data-testid={`todo-item-link-${todo.id}`}
        href={`/todos/${todo.id}`}
        aria-current={isSelected ? 'page' : undefined}
      >
        {todo.title}
      </a>
      <button
        type="button"
        data-testid={`todo-item-delete-${todo.id}`}
        onClick={() => deleteTodo(todo.id)}
      >
        x
      </button>
    </li>
  );
}
