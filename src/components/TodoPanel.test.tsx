import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TodoPanel } from './TodoPanel';
import { useServerStore } from '../stores/serverStore';

// Mock the store
vi.mock('../stores/serverStore', () => ({
  useServerStore: vi.fn(),
}));

describe('TodoPanel', () => {
  const mockLoadTodos = vi.fn().mockResolvedValue(undefined);
  const mockAddTodo = vi.fn().mockResolvedValue(undefined);
  const mockSetTodoCompleted = vi.fn().mockResolvedValue(undefined);
  const mockRemoveTodo = vi.fn().mockResolvedValue(undefined);
  
  const createMockStore = (projectId: string | null, todos: any[] = []) => {
    const todosByProject = projectId ? new Map([[projectId, todos]]) : new Map();
    return {
      todosByProject,
      getTodos: (id: string) => todosByProject.get(id) ?? [],
      loadTodos: mockLoadTodos,
      addTodo: mockAddTodo,
      setTodoCompleted: mockSetTodoCompleted,
      removeTodo: mockRemoveTodo,
    };
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should load todos for the correct project on mount', async () => {
    // Project not in store yet - should trigger load
    vi.mocked(useServerStore).mockReturnValue(createMockStore(null, []) as any);

    render(<TodoPanel projectId="project-1" projectName="Test Project" onViewAll={vi.fn()} />);

    await waitFor(() => {
      expect(mockLoadTodos).toHaveBeenCalledWith('project-1');
    });
  });

  it('should reload todos when project changes', async () => {
    // Start with project-1 loaded
    vi.mocked(useServerStore).mockReturnValue(createMockStore('project-1', []) as any);
    
    const { rerender } = render(
      <TodoPanel projectId="project-1" projectName="Project 1" onViewAll={vi.fn()} />
    );

    // Change to project-2 (not loaded yet)
    vi.mocked(useServerStore).mockReturnValue(createMockStore(null, []) as any);
    rerender(<TodoPanel projectId="project-2" projectName="Project 2" onViewAll={vi.fn()} />);

    await waitFor(() => {
      expect(mockLoadTodos).toHaveBeenCalledWith('project-2');
    });
  });

  it('should show loading state when todos have not been loaded for project', async () => {
    vi.mocked(useServerStore).mockReturnValue(createMockStore(null, []) as any);

    render(<TodoPanel projectId="project-2" projectName="Project 2" onViewAll={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText(/Loading todos for Project 2/i)).toBeInTheDocument();
    });
  });

  it('should display todos for the current project', () => {
    vi.mocked(useServerStore).mockReturnValue(createMockStore('project-1', [
      { id: '1', title: 'Todo 1', completed: false },
      { id: '2', title: 'Todo 2', completed: true },
    ]) as any);

    render(<TodoPanel projectId="project-1" projectName="Project 1" onViewAll={vi.fn()} />);

    expect(screen.getByText('Todo 1')).toBeInTheDocument();
    expect(screen.getByText('Todo 2')).toBeInTheDocument();
  });

  it('should call addTodo when adding a new todo', async () => {
    vi.mocked(useServerStore).mockReturnValue(createMockStore('project-1', []) as any);

    render(<TodoPanel projectId="project-1" projectName="Project 1" onViewAll={vi.fn()} />);

    const input = screen.getByPlaceholderText('Add a task...');
    fireEvent.change(input, { target: { value: 'New Todo' } });
    fireEvent.click(screen.getByText('+'));

    await waitFor(() => {
      expect(mockAddTodo).toHaveBeenCalledWith('project-1', 'New Todo');
    });
  });
  
  it('should show different todos for different projects', () => {
    // Project 1 has 2 todos, Project 2 has 1 todo
    const todosByProject = new Map([
      ['project-1', [{ id: '1', title: 'Project 1 Todo', completed: false }]],
      ['project-2', [{ id: '2', title: 'Project 2 Todo', completed: false }]],
    ]);
    
    vi.mocked(useServerStore).mockReturnValue({
      todosByProject,
      getTodos: (id: string) => todosByProject.get(id) ?? [],
      loadTodos: mockLoadTodos,
      addTodo: mockAddTodo,
      setTodoCompleted: mockSetTodoCompleted,
      removeTodo: mockRemoveTodo,
    } as any);

    const { rerender } = render(
      <TodoPanel projectId="project-1" projectName="Project 1" onViewAll={vi.fn()} />
    );

    expect(screen.getByText('Project 1 Todo')).toBeInTheDocument();
    expect(screen.queryByText('Project 2 Todo')).not.toBeInTheDocument();

    // Switch to project 2
    rerender(<TodoPanel projectId="project-2" projectName="Project 2" onViewAll={vi.fn()} />);

    expect(screen.queryByText('Project 1 Todo')).not.toBeInTheDocument();
    expect(screen.getByText('Project 2 Todo')).toBeInTheDocument();
  });
});
