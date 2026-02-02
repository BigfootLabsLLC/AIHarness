import { useEffect, useState } from 'react';
import { useServerStore } from '../stores/serverStore';
import { invoke } from '@tauri-apps/api/core';

interface TodoPanelProps {
  projectId: string;
  projectName: string;
  onViewAll: () => void;
}

export function TodoPanel({ projectId, projectName, onViewAll }: TodoPanelProps) {
  const { todosByProject, getTodos, loadTodos, addTodo, setTodoCompleted, removeTodo } = useServerStore();
  const [newTodoTitle, setNewTodoTitle] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  
  // Check if we've loaded todos for this project
  const hasLoaded = todosByProject.has(projectId);
  
  // Get todos for this specific project from the Map
  const todos = getTodos(projectId);

  // Reload todos when project changes or if data hasn't been loaded yet
  useEffect(() => {
    console.log('[TodoPanel] EFFECT RUNNING projectId:', projectId, 'Has loaded:', hasLoaded);
    // Write to file log
    invoke('debug_log_cmd', { msg: `[TodoPanel] projectId=${projectId} hasLoaded=${hasLoaded}` });
    if (!hasLoaded) {
      console.log('[TodoPanel] CALLING loadTodos with projectId:', projectId);
      invoke('debug_log_cmd', { msg: `[TodoPanel] CALLING loadTodos(${projectId})` });
      setIsLoading(true);
      Promise.resolve(loadTodos(projectId)).finally(() => setIsLoading(false));
    }
  }, [projectId, hasLoaded, loadTodos]);

  const handleAddTodo = async () => {
    if (!newTodoTitle.trim()) return;
    await addTodo(projectId, newTodoTitle.trim());
    setNewTodoTitle('');
  };

  if (!hasLoaded && isLoading) {
    return (
      <div className="p-2 text-sm text-amber-600">
        Loading todos for {projectName}...
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
            if (e.key === 'Enter') handleAddTodo();
          }}
        />
        <button className="todo-add-btn" onClick={handleAddTodo}>
          +
        </button>
      </div>

      {/* Loading indicator */}
      {isLoading && (
        <div className="text-xs text-gray-500 p-1">Loading...</div>
      )}

      {/* View all link */}
      {todos.length > 5 && (
        <button className="todo-view-all" onClick={onViewAll}>
          View all {todos.length} tasks →
        </button>
      )}

      {/* Todo list */}
      {todos.length === 0 ? (
        <div className="empty-state" style={{ fontSize: '11px' }}>
          No tasks yet for {projectName}.
        </div>
      ) : (
        todos.slice(0, 8).map((todo) => (
          <div key={todo.id} className="todo-item">
            <input
              type="checkbox"
              checked={todo.completed}
              onChange={() => setTodoCompleted(projectId, todo.id, !todo.completed)}
            />
            <span className={`todo-text ${todo.completed ? 'completed' : ''}`}>
              {todo.title}
            </span>
            <button
              className="todo-remove"
              onClick={() => removeTodo(projectId, todo.id)}
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
