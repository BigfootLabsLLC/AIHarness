import { useState } from 'react';
import { useCurrentProject } from '../contexts/ProjectContext';
import { useTodos } from '../hooks/useTodos';

interface TodoPanelSimpleProps {
  onViewAll: () => void;
}

export function TodoPanelSimple({ onViewAll }: TodoPanelSimpleProps) {
  const { currentProject } = useCurrentProject();
  const { todos, hasLoaded, add, setCompleted, remove } = useTodos();
  const [newTodoTitle, setNewTodoTitle] = useState('');

  const handleAdd = async () => {
    if (!newTodoTitle.trim()) return;
    await add(newTodoTitle.trim());
    setNewTodoTitle('');
  };

  if (!hasLoaded) {
    return (
      <div className="p-2 text-sm text-amber-600">
        Loading todos for {currentProject?.name ?? 'Unknown'}...
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2">
      {/* Add new todo */}
      <div className="todo-input-row">
        <input
          type="text"
          placeholder="Add a task..."
          className="todo-input"
          value={newTodoTitle}
          onChange={(e) => setNewTodoTitle(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') handleAdd();
          }}
        />
        <button className="todo-add-btn" onClick={handleAdd}>
          +
        </button>
      </div>

      {/* View all link */}
      {todos.length > 5 && (
        <button className="todo-view-all" onClick={onViewAll}>
          View all {todos.length} tasks →
        </button>
      )}

      {/* Todo list */}
      {todos.length === 0 ? (
        <div className="empty-state" style={{ fontSize: '11px' }}>
          No tasks yet for {currentProject?.name ?? 'Unknown'}.
        </div>
      ) : (
        todos.slice(0, 8).map((todo) => (
          <div key={todo.id} className="todo-item">
            <input
              type="checkbox"
              checked={todo.completed}
              onChange={() => setCompleted(todo.id, !todo.completed)}
            />
            <span className={`todo-text ${todo.completed ? 'completed' : ''}`}>
              {todo.title}
            </span>
            <button
              className="todo-remove"
              onClick={() => remove(todo.id)}
              title="Remove"
            >
              ×
            </button>
          </div>
        ))
      )}
    </div>
  );
}
