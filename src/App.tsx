import { useEffect, useMemo, useState } from 'react';
import type { ReactNode } from 'react';
import type { ToolCall } from './types';
import { useServerStore } from './stores/serverStore';

function App() {
  const { status, port, toolCalls, contextFiles, todos, projects, loadToolHistory, loadContextFilesForProject, loadTodos, createProject } = useServerStore();
  const [activeProject, setActiveProject] = useState('default');
  const [isProjectModalOpen, setIsProjectModalOpen] = useState(false);
  const [projectDraft, setProjectDraft] = useState({ name: '', rootPath: '' });
  const [projectError, setProjectError] = useState<string | null>(null);
  const filteredToolCalls = useMemo(
    () => toolCalls.filter((call) => call.project_id === activeProject),
    [toolCalls, activeProject],
  );
  const recentToolCalls = useMemo(() => filteredToolCalls.slice(0, 6), [filteredToolCalls]);
  const recentContext = useMemo(() => contextFiles.slice(0, 6), [contextFiles]);
  const [activeTab, setActiveTab] = useState<MainTab>({
    id: 'workspace',
    title: 'Workspace',
    kind: 'home',
  });
  const [tabs, setTabs] = useState<MainTab[]>([
    { id: 'workspace', title: 'Workspace', kind: 'home' },
  ]);

  useEffect(() => {
    useServerStore.getState().initialize();
  }, []);

  useEffect(() => {
    if (projects.length > 0 && !projects.some((p) => p.id === activeProject)) {
      setActiveProject(projects[0].id);
    }
  }, [projects, activeProject]);

  useEffect(() => {
    loadToolHistory(activeProject);
    loadContextFilesForProject(activeProject);
    loadTodos(activeProject);
  }, [activeProject, loadToolHistory, loadContextFilesForProject, loadTodos]);

  useEffect(() => {
    if (toolCalls.length === 0) return;
    const latest = toolCalls[0];
    if (latest.project_id === activeProject && latest.tool_name.startsWith('todo_')) {
      loadTodos(activeProject);
    }
  }, [toolCalls, activeProject, loadTodos]);

  const openProjectModal = () => {
    setProjectDraft({ name: '', rootPath: '' });
    setProjectError(null);
    setIsProjectModalOpen(true);
  };

  const closeProjectModal = () => {
    setIsProjectModalOpen(false);
    setProjectError(null);
  };

  const submitProject = async () => {
    if (!projectDraft.name.trim()) {
      setProjectError('Project name is required.');
      return;
    }
    if (!projectDraft.rootPath.trim()) {
      setProjectError('Project folder path is required.');
      return;
    }
    const project = await createProject(projectDraft.name.trim(), projectDraft.rootPath.trim());
    if (!project) {
      setProjectError('Failed to create project.');
      return;
    }
    setActiveProject(project.id);
    closeProjectModal();
  };

  return (
    <div className="min-h-screen app-shell">
      <header className="app-topbar">
        <div className="app-topbar__left">
          <div className="brand-lockup">
            <div className="brand-mark">AI</div>
            <div>
              <div className="brand-title">AIHarness</div>
              <div className="brand-subtitle">Context control center</div>
            </div>
          </div>
          <div className="status-chip">
            <span className={`status-dot status-${status}`} />
            <span className="status-text">{status}</span>
            <span className="status-port">:{port}</span>
          </div>
        </div>
        <div className="app-topbar__actions" />
      </header>

      <div className="app-body">
        <aside className="project-rail">
          <div className="rail-label">Projects</div>
          {projects.map((project) => (
            <button
              key={project.id}
              className={`project-tab ${activeProject === project.id ? 'active' : ''}`}
              onClick={() => setActiveProject(project.id)}
            >
              {project.name}
            </button>
          ))}
          <button
            className="project-tab add"
            onClick={openProjectModal}
          >
            ＋
          </button>
        </aside>

        <div className="workspace">
          <section className="workspace-left">
            <PanelShell title="Tool Use History" tabs={['History', 'Tools']}>
              <div className="stack">
                {recentToolCalls.length === 0 ? (
                  <div className="empty-state">No tool calls yet.</div>
                ) : (
                  recentToolCalls.map((call) => (
                    <button
                      key={call.id}
                      className="list-row compact-row"
                      onClick={() => {
                        const nextTab = tabFromToolCall(call);
                        setTabs((prev) => addOrReplaceTab(prev, nextTab));
                        setActiveTab(nextTab);
                      }}
                    >
                      <div>
                        <div className="row-title">{call.tool_name}</div>
                        <div className="row-subtitle">
                          {call.success ? 'Succeeded' : 'Failed'} · {new Date(call.timestamp).toLocaleTimeString()}
                        </div>
                      </div>
                      <div className="row-meta">{call.duration_ms}ms</div>
                    </button>
                  ))
                )}
              </div>
            </PanelShell>

            <PanelShell title="File System Tree" tabs={['Files', 'Tags']}>
              <div className="stack">
                <div className="empty-state">No files loaded.</div>
              </div>
            </PanelShell>

            <PanelShell title="Todo Queue" tabs={['Active', 'Backlog']}>
              <div className="stack">
                {todos.length === 0 ? (
                  <div className="empty-state">No tasks yet.</div>
                ) : (
                  todos.map((todo) => (
                    <label key={todo.id} className="todo-row compact-row">
                      <input
                        type="checkbox"
                        checked={todo.completed}
                        onChange={() => {
                          useServerStore.getState().setTodoCompleted(activeProject, todo.id, !todo.completed);
                        }}
                      />
                      <div>
                        <div className="row-title">{todo.title}</div>
                        {todo.description && <div className="row-subtitle">{todo.description}</div>}
                      </div>
                    </label>
                  ))
                )}
              </div>
            </PanelShell>
          </section>

          <section className="workspace-center">
            <div className="main-canvas">
              {activeTab.kind === 'tool' ? (
                <ToolCallDetail call={activeTab.payload} />
              ) : (
                <div className="canvas-empty">
                  <div className="canvas-title">Main Canvas</div>
                  <p>Open a file, architecture map, or settings view to focus here.</p>
                </div>
              )}
            </div>
            <div className="canvas-tabs">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  className={`canvas-tab ${tab.id === activeTab.id ? 'active' : ''}`}
                  onClick={() => setActiveTab(tab)}
                >
                  {tab.title}
                </button>
              ))}
            </div>
          </section>

          <section className="workspace-right">
            <PanelShell title="Questions from Agents" tabs={['Inbox', 'Resolved']}>
              <div className="stack">
                <div className="empty-state">No questions yet.</div>
              </div>
            </PanelShell>

            <PanelShell title="Context Snapshot" tabs={['Context', 'Notes']}>
              <div className="stack">
                {recentContext.length === 0 ? (
                  <div className="empty-state">No context files yet.</div>
                ) : (
                  recentContext.map((file) => (
                    <div key={file.id} className="list-row">
                      <div>
                        <div className="row-title">{file.path.split('/').pop()}</div>
                        <div className="row-subtitle">{file.path}</div>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </PanelShell>

            <PanelShell title="Agents" tabs={['Live', 'Paused']}>
              <div className="stack">
                <div className="empty-state">No agents active.</div>
              </div>
            </PanelShell>
          </section>
        </div>
      </div>

      {isProjectModalOpen && (
        <ProjectCreateModal
          name={projectDraft.name}
          rootPath={projectDraft.rootPath}
          error={projectError}
          onChangeName={(value) => setProjectDraft((prev) => ({ ...prev, name: value }))}
          onChangeRootPath={(value) => setProjectDraft((prev) => ({ ...prev, rootPath: value }))}
          onCancel={closeProjectModal}
          onSubmit={submitProject}
        />
      )}
    </div>
  );
}

export default App;

function PanelShell({
  title,
  tabs,
  children,
}: {
  title: string;
  tabs: string[];
  children: ReactNode;
}) {
  return (
    <div className="panel-shell">
      <div className="panel-shell__header">
        <div>
          <div className="panel-title">{title}</div>
          <div className="panel-tabs">
            {tabs.map((tab, index) => (
              <button key={tab} className={`panel-tab ${index === 0 ? 'active' : ''}`}>
                {tab}
              </button>
            ))}
          </div>
        </div>
        <button className="panel-action">⋯</button>
      </div>
      <div className="panel-shell__body">{children}</div>
    </div>
  );
}

type MainTab = {
  id: string;
  title: string;
  kind: 'home' | 'tool';
  payload?: ToolCall;
};

function tabFromToolCall(call: ToolCall): MainTab {
  return {
    id: `tool-${call.id}`,
    title: call.tool_name,
    kind: 'tool',
    payload: call,
  };
}

function addOrReplaceTab(existing: MainTab[], next: MainTab): MainTab[] {
  const hasTab = existing.some((tab) => tab.id === next.id);
  if (hasTab) {
    return existing.map((tab) => (tab.id === next.id ? next : tab));
  }
  return [...existing, next];
}

function ToolCallDetail({ call }: { call?: ToolCall }) {
  if (!call) {
    return <div className="canvas-empty">No tool call selected.</div>;
  }

  return (
    <div className="detail-card">
      <div className="detail-header">
        <div>
          <div className="detail-title">{call.tool_name}</div>
          <div className="detail-subtitle">
            {call.success ? 'Succeeded' : 'Failed'} · {new Date(call.timestamp).toLocaleString()}
          </div>
        </div>
        <div className="detail-meta">{call.duration_ms}ms</div>
      </div>
      <div className="detail-section">
        <div className="detail-label">Arguments</div>
        <pre className="detail-code">{JSON.stringify(call.arguments, null, 2)}</pre>
      </div>
      <div className="detail-section">
        <div className="detail-label">Result</div>
        <pre className="detail-code">{call.content}</pre>
      </div>
    </div>
  );
}

function ProjectCreateModal({
  name,
  rootPath,
  error,
  onChangeName,
  onChangeRootPath,
  onCancel,
  onSubmit,
}: {
  name: string;
  rootPath: string;
  error: string | null;
  onChangeName: (value: string) => void;
  onChangeRootPath: (value: string) => void;
  onCancel: () => void;
  onSubmit: () => void;
}) {
  return (
    <div className="modal-backdrop" onClick={onCancel}>
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="modal-title">Create project</div>
        <label className="field">
          <span className="field-label">Name</span>
          <input
            className="field-input"
            value={name}
            onChange={(event) => onChangeName(event.target.value)}
            placeholder="Project name"
            autoFocus
          />
        </label>
        <label className="field">
          <span className="field-label">Folder</span>
          <input
            className="field-input"
            value={rootPath}
            onChange={(event) => onChangeRootPath(event.target.value)}
            placeholder="/path/to/project"
          />
        </label>
        {error && <div className="field-error">{error}</div>}
        <div className="modal-actions">
          <button className="button" onClick={onCancel}>
            Cancel
          </button>
          <button className="button primary" onClick={onSubmit}>
            Create
          </button>
        </div>
      </div>
    </div>
  );
}
