import { useEffect, useMemo, useState } from 'react';
import type { ReactNode } from 'react';
import type { BuildCommand, DirectoryEntry, DirectoryListing, ToolCall } from './types';
import { useServerStore } from './stores/serverStore';
import { open } from '@tauri-apps/plugin-dialog';

function App() {
  const {
    status,
    port,
    toolCalls,
    contextFiles,
    contextNotes,
    todos,
    buildCommands,
    projects,
    loadToolHistory,
    loadContextFilesForProject,
    loadContextNotes,
    loadTodos,
    createProject,
    listProjectDirectory,
    listDirectory,
    addContextFile,
    addContextNote,
    updateContextNote,
    removeContextNote,
    loadBuildCommands,
    runBuildCommand,
    getDefaultBuildCommand,
    resetProjectData,
    executeTool,
    setCurrentProject,
  } = useServerStore();
  const [activeProject, setActiveProject] = useState('default');
  const [isProjectModalOpen, setIsProjectModalOpen] = useState(false);
  const [projectDraft, setProjectDraft] = useState({ name: '', rootPath: '' });
  const [projectError, setProjectError] = useState<string | null>(null);
  const [filePanelTab, setFilePanelTab] = useState<'project' | 'system'>('project');
  const [contextPanelTab, setContextPanelTab] = useState<'context' | 'notes'>('context');
  const [projectTree, setProjectTree] = useState<Record<string, DirectoryListing>>({});
  const [projectExpanded, setProjectExpanded] = useState<Record<string, boolean>>({});
  const [systemTree, setSystemTree] = useState<Record<string, DirectoryListing>>({});
  const [systemExpanded, setSystemExpanded] = useState<Record<string, boolean>>({});
  const [systemRoot, setSystemRoot] = useState('');
  const [newNote, setNewNote] = useState('');
  const [defaultBuild, setDefaultBuild] = useState<BuildCommand | null>(null);
  const [buildOutputsByProject, setBuildOutputsByProject] = useState<Record<string, BuildOutputItem[]>>({});
  const filteredToolCalls = useMemo(
    () => toolCalls.filter((call) => call.project_id === activeProject),
    [toolCalls, activeProject],
  );
  const recentToolCalls = useMemo(() => filteredToolCalls.slice(0, 6), [filteredToolCalls]);
  const recentContext = useMemo(() => contextFiles.slice(0, 6), [contextFiles]);
  const activeProjectInfo = useMemo(
    () => projects.find((project) => project.id === activeProject) ?? null,
    [projects, activeProject],
  );
  const buildOutputs = useMemo(
    () => buildOutputsByProject[activeProject] ?? [],
    [buildOutputsByProject, activeProject],
  );
  const statusClass =
    status === 'running'
      ? 'status-running'
      : status === 'starting'
        ? 'status-starting'
        : status === 'error'
          ? 'status-error'
          : '';
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
    setDefaultBuild(null);
    setTabs([{ id: 'workspace', title: 'Workspace', kind: 'home' }]);
    setActiveTab({ id: 'workspace', title: 'Workspace', kind: 'home' });
    setCurrentProject(activeProject);
    resetProjectData();
    loadToolHistory(activeProject);
    loadContextFilesForProject(activeProject);
    loadContextNotes(activeProject);
    loadTodos(activeProject);
    loadBuildCommands(activeProject);
    getDefaultBuildCommand(activeProject).then(setDefaultBuild);
  }, [
    activeProject,
    loadToolHistory,
    loadContextFilesForProject,
    loadContextNotes,
    loadTodos,
    loadBuildCommands,
    getDefaultBuildCommand,
    resetProjectData,
    setCurrentProject,
  ]);

  useEffect(() => {
    if (toolCalls.length === 0) return;
    const latest = toolCalls[0];
    if (latest.project_id === activeProject && latest.tool_name.startsWith('todo_')) {
      loadTodos(activeProject);
    }
  }, [toolCalls, activeProject, loadTodos]);

  useEffect(() => {
    if (!activeProjectInfo) return;
    const root = activeProjectInfo.root_path;
    setProjectTree({});
    setProjectExpanded({ [root]: true });
    listProjectDirectory(activeProject, '').then((listing) => {
      if (listing) {
        setProjectTree({ [listing.path]: listing });
      }
    });
  }, [activeProjectInfo, activeProject, listProjectDirectory]);

  useEffect(() => {
    if (activeProjectInfo?.root_path) {
      setSystemRoot(activeProjectInfo.root_path);
    }
  }, [activeProjectInfo]);

  useEffect(() => {
    if (!systemRoot) return;
    setSystemTree({});
    setSystemExpanded({ [systemRoot]: true });
    listDirectory(systemRoot).then((listing) => {
      if (listing) {
        setSystemTree({ [listing.path]: listing });
      }
    });
  }, [systemRoot, listDirectory]);

  useEffect(() => {
    const activeDefault = buildCommands.find((cmd) => cmd.is_default) ?? null;
    if (activeDefault) {
      setDefaultBuild(activeDefault);
      return;
    }
    if (buildCommands.length > 0 && !defaultBuild) {
      setDefaultBuild(buildCommands[0]);
    }
  }, [buildCommands, defaultBuild]);

  const openFile = async (entry: DirectoryEntry) => {
    const content = await executeTool('read_file', { path: entry.path }, activeProject);
    const nextTab: MainTab = {
      id: `file-${entry.path}`,
      title: entry.name,
      kind: 'file',
      payload: {
        path: entry.path,
        content,
      },
    };
    setTabs((prev) => addOrReplaceTab(prev, nextTab));
    setActiveTab(nextTab);
  };

  const toggleProjectEntry = async (entry: DirectoryEntry) => {
    if (!activeProjectInfo) return;
    if (!entry.is_dir) {
      void openFile(entry);
      return;
    }
    setProjectExpanded((prev) => ({ ...prev, [entry.path]: !prev[entry.path] }));
    if (!projectTree[entry.path]) {
      const subPath = relativeSubPath(activeProjectInfo.root_path, entry.path);
      const listing = await listProjectDirectory(activeProject, subPath);
      if (listing) {
        setProjectTree((prev) => ({ ...prev, [listing.path]: listing }));
      }
    }
  };

  const toggleSystemEntry = async (entry: DirectoryEntry) => {
    if (!entry.is_dir) {
      void openFile(entry);
      return;
    }
    setSystemExpanded((prev) => ({ ...prev, [entry.path]: !prev[entry.path] }));
    if (!systemTree[entry.path]) {
      const listing = await listDirectory(entry.path);
      if (listing) {
        setSystemTree((prev) => ({ ...prev, [listing.path]: listing }));
      }
    }
  };

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
    const result = await createProject(projectDraft.name.trim(), projectDraft.rootPath.trim());
    if (!result.project) {
      setProjectError(result.error ?? 'Failed to create project.');
      return;
    }
    setActiveProject(result.project.id);
    closeProjectModal();
  };

  const startBuildOutput = (projectId: string, tab: MainTab, payload: BuildPayload) => {
    setBuildOutputsByProject((prev) => {
      const current = prev[projectId] ?? [];
      const next: BuildOutputItem = {
        id: tab.id,
        name: payload.name,
        command: payload.command,
        output: payload.output,
        timestamp: Date.now(),
        status: 'running',
      };
      const remaining = current.filter((item) => item.id !== tab.id);
      return {
        ...prev,
        [projectId]: [next, ...remaining].slice(0, 8),
      };
    });
  };

  const finalizeBuildOutput = (projectId: string, tab: MainTab, payload: BuildPayload, status: BuildOutputStatus) => {
    setTabs((prev) => addOrReplaceTab(prev, tab));
    setActiveTab(tab);
    setBuildOutputsByProject((prev) => {
      const current = prev[projectId] ?? [];
      const next: BuildOutputItem = {
        id: tab.id,
        name: payload.name,
        command: payload.command,
        output: payload.output,
        timestamp: Date.now(),
        status,
      };
      const remaining = current.filter((item) => item.id !== tab.id);
      return {
        ...prev,
        [projectId]: [next, ...remaining].slice(0, 8),
      };
    });
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
            <span className={`status-dot ${statusClass}`} />
            <span className="status-text">{status}</span>
            <span className="status-port">:{port}</span>
          </div>
          {activeProjectInfo && (
            <div className="project-chip">
              <span className="project-chip__label">Project</span>
              <span className="project-chip__name">{activeProjectInfo.name}</span>
            </div>
          )}
        </div>
        <div className="app-topbar__actions">
          <button
            className="toolbar-button"
            disabled={!defaultBuild}
            onClick={async () => {
              if (!defaultBuild) return;
              const payload: BuildPayload = {
                name: defaultBuild.name,
                command: defaultBuild.command,
                output: 'Running build...',
              };
              const nextTab: MainTab = {
                id: `build-${defaultBuild.id}-${Date.now()}`,
                title: defaultBuild.name,
                kind: 'build',
                payload,
              };
              startBuildOutput(activeProject, nextTab, payload);
              const output = await runBuildCommand(activeProject, defaultBuild.id);
              const finalPayload: BuildPayload = {
                name: defaultBuild.name,
                command: defaultBuild.command,
                output: output ?? 'Build failed or no output returned.',
              };
              finalizeBuildOutput(activeProject, nextTab, finalPayload, output ? 'done' : 'failed');
            }}
          >
            Build
          </button>
        </div>
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

            <PanelShell
              title="File System Tree"
              tabs={['Project', 'System']}
              activeTab={filePanelTab === 'project' ? 'Project' : 'System'}
              onTabChange={(tab) => setFilePanelTab(tab === 'Project' ? 'project' : 'system')}
            >
              <div className="stack">
                {filePanelTab === 'project' ? (
                  <FileTree
                    rootPath={activeProjectInfo?.root_path ?? ''}
                    listings={projectTree}
                    expanded={projectExpanded}
                    onToggle={toggleProjectEntry}
                    onAddToContext={(entry) => addContextFile(activeProject, entry.path)}
                  />
                ) : (
                  <FileTree
                    rootPath={systemRoot}
                    listings={systemTree}
                    expanded={systemExpanded}
                    onToggle={toggleSystemEntry}
                    onAddToContext={(entry) => addContextFile(activeProject, entry.path)}
                  />
                )}
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
                <ToolCallDetail call={activeTab.payload as ToolCall} />
              ) : activeTab.kind === 'file' ? (
                <FileDetail payload={activeTab.payload as FilePayload} />
              ) : activeTab.kind === 'build' ? (
                <BuildOutputDetail payload={activeTab.payload as BuildPayload} />
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

            <PanelShell title="Command Output" tabs={['Builds']}>
              <div className="stack">
                {buildOutputs.length === 0 ? (
                  <div className="empty-state">No build output yet.</div>
                ) : (
                  buildOutputs.map((item) => (
                    <button
                      key={item.id}
                      className="list-row compact-row"
                      onClick={() => {
                        const nextTab: MainTab = {
                          id: item.id,
                          title: item.name,
                          kind: 'build',
                          payload: {
                            name: item.name,
                            command: item.command,
                            output: item.output,
                          },
                        };
                        setTabs((prev) => addOrReplaceTab(prev, nextTab));
                        setActiveTab(nextTab);
                      }}
                    >
                      <div>
                        <div className="row-title">{item.name}</div>
                        <div className="row-subtitle">{previewOutput(item)}</div>
                      </div>
                      <div className="row-meta">
                        {item.status} · {new Date(item.timestamp).toLocaleTimeString()}
                      </div>
                    </button>
                  ))
                )}
              </div>
            </PanelShell>

            <PanelShell
              title="Context Snapshot"
              tabs={['Context', 'Notes']}
              activeTab={contextPanelTab === 'context' ? 'Context' : 'Notes'}
              onTabChange={(tab) => setContextPanelTab(tab === 'Context' ? 'context' : 'notes')}
            >
              <div className="stack">
                {contextPanelTab === 'context' ? (
                  recentContext.length === 0 ? (
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
                  )
                ) : (
                  <div className="stack">
                    <div className="note-input">
                      <input
                        value={newNote}
                        onChange={(event) => setNewNote(event.target.value)}
                        placeholder="Add context note..."
                      />
                      <button
                        onClick={() => {
                          if (!newNote.trim()) return;
                          void addContextNote(activeProject, newNote.trim());
                          setNewNote('');
                        }}
                      >
                        Add
                      </button>
                    </div>
                    {contextNotes.length === 0 ? (
                      <div className="empty-state">No notes yet.</div>
                    ) : (
                      contextNotes.map((note) => (
                        <div key={note.id} className="note-row">
                          <textarea
                            defaultValue={note.content}
                            onBlur={(event) => {
                              const value = event.target.value.trim();
                              updateContextNote(activeProject, note.id, value).catch(() => undefined);
                            }}
                          />
                          <button onClick={() => removeContextNote(activeProject, note.id)}>Remove</button>
                        </div>
                      ))
                    )}
                  </div>
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
  onTabChange,
  activeTab,
}: {
  title: string;
  tabs: string[];
  children: ReactNode;
  onTabChange?: (tab: string) => void;
  activeTab?: string;
}) {
  return (
    <div className="panel-shell">
      <div className="panel-shell__header">
        <div>
          <div className="panel-title">{title}</div>
          <div className="panel-tabs">
            {tabs.map((tab, index) => (
              <button
                key={tab}
                className={`panel-tab ${activeTab ? (tab === activeTab ? 'active' : '') : index === 0 ? 'active' : ''}`}
                onClick={() => onTabChange?.(tab)}
              >
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
  kind: 'home' | 'tool' | 'file' | 'build';
  payload?: ToolCall | FilePayload | BuildPayload;
};

function tabFromToolCall(call: ToolCall): MainTab {
  return {
    id: `tool-${call.id}`,
    title: call.tool_name,
    kind: 'tool',
    payload: call,
  };
}

type FilePayload = {
  path: string;
  content: string;
};

type BuildPayload = {
  name: string;
  command: string;
  output: string;
};

type BuildOutputStatus = 'running' | 'done' | 'failed';

type BuildOutputItem = BuildPayload & {
  id: string;
  timestamp: number;
  status: BuildOutputStatus;
};

function addOrReplaceTab(existing: MainTab[], next: MainTab): MainTab[] {
  const hasTab = existing.some((tab) => tab.id === next.id);
  if (hasTab) {
    return existing.map((tab) => (tab.id === next.id ? next : tab));
  }
  return [...existing, next];
}

function previewOutput(item: BuildOutputItem): string {
  if (item.status === 'running') return 'Running...';
  const trimmed = item.output.trim();
  if (!trimmed) return 'No output.';
  const lines = trimmed.split('\n').map((line) => line.trim()).filter(Boolean);
  const signalLine = lines.find((line) => /error|failed|panic|warning/i.test(line));
  const candidate = signalLine ?? lines[0] ?? trimmed;
  return candidate.slice(0, 140);
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

function FileDetail({ payload }: { payload?: FilePayload }) {
  if (!payload) {
    return <div className="canvas-empty">No file selected.</div>;
  }

  return (
    <div className="detail-card">
      <div className="detail-header">
        <div>
          <div className="detail-title">{payload.path.split('/').pop()}</div>
          <div className="detail-subtitle">{payload.path}</div>
        </div>
      </div>
      <div className="detail-section">
        <div className="detail-label">Content</div>
        <pre className="detail-code">{payload.content}</pre>
      </div>
    </div>
  );
}

function BuildOutputDetail({ payload }: { payload?: BuildPayload }) {
  if (!payload) {
    return <div className="canvas-empty">No build output.</div>;
  }

  return (
    <div className="detail-card">
      <div className="detail-header">
        <div>
          <div className="detail-title">{payload.name}</div>
          <div className="detail-subtitle">{payload.command}</div>
        </div>
      </div>
      <div className="detail-section">
        <div className="detail-label">Output</div>
        <pre className="detail-code">{payload.output}</pre>
      </div>
    </div>
  );
}

function FileTree({
  rootPath,
  listings,
  expanded,
  onToggle,
  onAddToContext,
}: {
  rootPath: string;
  listings: Record<string, DirectoryListing>;
  expanded: Record<string, boolean>;
  onToggle: (entry: DirectoryEntry) => void;
  onAddToContext: (entry: DirectoryEntry) => void;
}) {
  if (!rootPath) {
    return <div className="empty-state">No project selected.</div>;
  }

  const rootListing = listings[rootPath];
  if (!rootListing) {
    return <div className="empty-state">Loading tree...</div>;
  }

  const renderNode = (path: string, depth: number) => {
    const listing = listings[path];
    if (!listing) return null;

    return listing.entries.map((entry) => {
      const isExpanded = !!expanded[entry.path];
      const hasChildren = entry.is_dir;
      return (
        <div key={entry.path}>
          <div
            className="tree-row"
            style={{ paddingLeft: `${8 + depth * 12}px` }}
          >
            <button className="tree-toggle" onClick={() => onToggle(entry)}>
              <span className="tree-icon">{hasChildren ? (isExpanded ? '▾' : '▸') : '·'}</span>
              <span className="tree-name">{entry.name}</span>
            </button>
            {!entry.is_dir && (
              <button className="file-action" onClick={() => onAddToContext(entry)}>
                Add
              </button>
            )}
          </div>
          {entry.is_dir && isExpanded && renderNode(entry.path, depth + 1)}
        </div>
      );
    });
  };

  return <div className="tree-view">{renderNode(rootPath, 0)}</div>;
}

function relativeSubPath(rootPath: string, entryPath: string): string {
  if (!entryPath.startsWith(rootPath)) return '';
  const trimmed = entryPath.slice(rootPath.length);
  if (trimmed.startsWith('/')) {
    return trimmed.slice(1);
  }
  return trimmed;
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
          <div className="field-row">
            <input
              className="field-input"
              value={rootPath}
              onChange={(event) => onChangeRootPath(event.target.value)}
              placeholder="/path/to/project"
            />
            <button
              className="button"
              type="button"
              onClick={async () => {
                const selection = await open({ directory: true, multiple: false });
                if (typeof selection === 'string') {
                  onChangeRootPath(selection);
                }
              }}
            >
              Browse
            </button>
          </div>
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
