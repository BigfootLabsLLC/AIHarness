import { render, screen } from '@testing-library/react';
import App from './App';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'get_server_status') {
      return { running: true, port: 8787 };
    }
    if (cmd === 'get_event_history') {
      return [];
    }
    if (cmd === 'list_projects') {
      return [];
    }
    if (cmd === 'list_context_files') {
      return [];
    }
    if (cmd === 'get_home_directory') {
      return '/home/user';
    }
    if (cmd === 'list_directory') {
      return { path: '/home/user', entries: [], is_dir: true };
    }
    if (cmd === 'list_todos') {
      return [];
    }
    if (cmd === 'list_build_commands') {
      return [];
    }
    if (cmd === 'get_default_build_command') {
      return null;
    }
    if (cmd === 'list_context_notes') {
      return [];
    }
    return null;
  }),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the AIHarness title', () => {
    render(<App />);
    expect(screen.getByText('AIHarness')).toBeInTheDocument();
    expect(screen.getByText('Context control center')).toBeInTheDocument();
  });

  it('renders the Todo Queue panel', () => {
    render(<App />);
    // The panel title might include the count, e.g., "Todo Queue (0)"
    expect(screen.getByText(/Todo Queue/)).toBeInTheDocument();
  });
});
