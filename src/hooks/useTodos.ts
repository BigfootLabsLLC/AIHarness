import { useCallback, useMemo } from 'react';
import { useServerStore } from '../stores/serverStore';
import { useCurrentProject } from '../contexts/ProjectContext';

export function useTodos() {
  const { currentProjectId } = useCurrentProject();
  const { todosByProject, addTodo, setTodoCompleted, removeTodo, loadTodos } = useServerStore();

  // Automatically get todos for current project
  const todos = useMemo(() => {
    return todosByProject.get(currentProjectId) ?? [];
  }, [todosByProject, currentProjectId]);

  // Check if loaded
  const hasLoaded = todosByProject.has(currentProjectId);

  // Project-scoped actions - these automatically use current project
  const add = useCallback((title: string, description?: string) => {
    return addTodo(currentProjectId, title, description);
  }, [currentProjectId, addTodo]);

  const setCompleted = useCallback((id: string, completed: boolean) => {
    return setTodoCompleted(currentProjectId, id, completed);
  }, [currentProjectId, setTodoCompleted]);

  const remove = useCallback((id: string) => {
    return removeTodo(currentProjectId, id);
  }, [currentProjectId, removeTodo]);

  const reload = useCallback(() => {
    return loadTodos(currentProjectId);
  }, [currentProjectId, loadTodos]);

  return {
    todos,
    hasLoaded,
    activeCount: todos.filter(t => !t.completed).length,
    completedCount: todos.filter(t => t.completed).length,
    add,
    setCompleted,
    remove,
    reload
  };
}
