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

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should load todos for the correct project on mount', async () => {
    vi.mocked(useServerStore).mockReturnValue({
      todos: [],
      todosProjectId: 'project-1',
      loadTodos: mockLoadTodos,
      addTodo: mockAddTodo,
      setTodoCompleted: mockSetTodoCompleted,
      removeTodo: mockRemoveTodo,
    } as any);

    render(<TodoPanel projectId="project-1" projectName="Test Project" onViewAll={vi.fn()} />);

    await waitFor(() => {
      expect(mockLoadTodos).toHaveBeenCalledWith('project-1');
    });
  });

  it('should reload todos when project changes', async () => {
    const { rerender } = render(
      <TodoPanel projectId="project-1" projectName="Project 1" onViewAll={vi.fn()} />
    );

    vi.mocked(useServerStore).mockReturnValue({
      todos: [],
      todosProjectId: 'project-1',
      loadTodos: mockLoadTodos,
      addTodo: mockAddTodo,
      setTodoCompleted: mockSetTodoCompleted,
      removeTodo: mockRemoveTodo,
    } as any);

    // Change to project-2
    rerender(<TodoPanel projectId="project-2" projectName="Project 2" onViewAll={vi.fn()} />);

    await waitFor(() => {
      expect(mockLoadTodos).toHaveBeenCalledWith('project-2');
    });
  });

  it('should show loading state when todos do not match current project', async () => {
    vi.mocked(useServerStore).mockReturnValue({
      todos: [{ id: '1', title: 'Old Todo', completed: false }],
      todosProjectId: 'project-1', // Stale data from different project
      loadTodos: mockLoadTodos,
      addTodo: mockAddTodo,
      setTodoCompleted: mockSetTodoCompleted,
      removeTodo: mockRemoveTodo,
    } as any);

    render(<TodoPanel projectId="project-2" projectName="Project 2" onViewAll={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText(/Loading todos for Project 2/i)).toBeInTheDocument();
    });
  });

  it('should display todos for the current project', () => {
    vi.mocked(useServerStore).mockReturnValue({
      todos: [
        { id: '1', title: 'Todo 1', completed: false },
        { id: '2', title: 'Todo 2', completed: true },
      ],
      todosProjectId: 'project-1', // Matches current project
      loadTodos: mockLoadTodos,
      addTodo: mockAddTodo,
      setTodoCompleted: mockSetTodoCompleted,
      removeTodo: mockRemoveTodo,
    } as any);

    render(<TodoPanel projectId="project-1" projectName="Project 1" onViewAll={vi.fn()} />);

    expect(screen.getByText('Todo 1')).toBeInTheDocument();
    expect(screen.getByText('Todo 2')).toBeInTheDocument();
  });

  it('should call addTodo when adding a new todo', async () => {
    vi.mocked(useServerStore).mockReturnValue({
      todos: [],
      todosProjectId: 'project-1',
      loadTodos: mockLoadTodos,
      addTodo: mockAddTodo,
      setTodoCompleted: mockSetTodoCompleted,
      removeTodo: mockRemoveTodo,
    } as any);

    render(<TodoPanel projectId="project-1" projectName="Project 1" onViewAll={vi.fn()} />);

    const input = screen.getByPlaceholderText('Add a task...');
    fireEvent.change(input, { target: { value: 'New Todo' } });
    fireEvent.click(screen.getByText('+'));

    await waitFor(() => {
      expect(mockAddTodo).toHaveBeenCalledWith('project-1', 'New Todo');
    });
  });
});
