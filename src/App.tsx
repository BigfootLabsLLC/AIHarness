import { useEffect, useMemo, useState } from 'react';
import type { ReactNode } from 'react';
import type { BuildCommand, DirectoryEntry, DirectoryListing, ToolCall, McpConfigResult, McpToolInfo } from './types';
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
    getHomeDirectory,
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
    getMcpSupportedTools,
    configureMcpForTool,
    configureMcpForAllTools,
  } = useServerStore();
  const [activeProject, setActiveProjectState] = useState('default');
  
  // Persist active project to localStorage
  const setActiveProject = (projectId: string) => {
    setActiveProjectState(projectId);
    localStorage.setItem('aiharness_last_project', projectId);
  };
  
  // Restore last project on mount
  useEffect(() => {
    const saved = localStorage.getItem('aiharness_last_project');
    if (saved) {
      setActiveProjectState(saved);
    }
  }, []);
  const [isProjectModalOpen, setIsProjectModalOpen] = useState(false);
  const [projectDraft, setProjectDraft] = useState({ name: '', rootPath: '' });
  const [projectError, setProjectError] = useState<string | null>(null);
  const [filePanelTab, setFilePanelTab] = useState<'project' | 'system'>('project');
  const [showHiddenFiles, setShowHiddenFiles] = useState(false);
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
  // MCP Config modal state
  const [isMcpModalOpen, setIsMcpModalOpen] = useState(false);
  const [mcpTools, setMcpTools] = useState<McpToolInfo[]>([]);
  const [mcpResults, setMcpResults] = useState<McpConfigResult[]>([]);
  const [isMcpConfiguring, setIsMcpConfiguring] = useState(false);

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

  // System file browser starts at home directory
  useEffect(() => {
    getHomeDirectory().then((home) => {
      if (home) {
        setSystemRoot(home);
      }
    });
  }, [getHomeDirectory]);

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

  const navigateSystemUp = async () => {
    const currentListing = systemTree[systemRoot];
    if (currentListing?.parent_path) {
      const parentPath = currentListing.parent_path;
      setSystemRoot(parentPath);
      const listing = await listDirectory(parentPath);
      if (listing) {
        setSystemTree({ [listing.path]: listing });
        setSystemExpanded({ [parentPath]: true });
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
            <div className="project-chip" title={activeProjectInfo.root_path}>
              <span className="project-chip__label">Project</span>
              <span className="project-chip__name">{activeProjectInfo.name}</span>
              <span className="project-chip__path">{activeProjectInfo.root_path}</span>
            </div>
          )}
        </div>
        <div className="app-topbar__actions">
          <button
            className="toolbar-button"
            disabled={!activeProjectInfo}
            onClick={async () => {
              if (!activeProjectInfo) return;
              const tools = await getMcpSupportedTools();
              setMcpTools(tools);
              setMcpResults([]);
              setIsMcpModalOpen(true);
            }}
          >
            Add MCP
          </button>
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
                    showParent={false}
                    showHidden={showHiddenFiles}
                    onToggleHidden={() => setShowHiddenFiles(!showHiddenFiles)}
                  />
                ) : (
                  <FileTree
                    rootPath={systemRoot}
                    listings={systemTree}
                    expanded={systemExpanded}
                    onToggle={toggleSystemEntry}
                    onAddToContext={(entry) => addContextFile(activeProject, entry.path)}
                    onNavigateUp={navigateSystemUp}
                    showParent={true}
                    showHidden={showHiddenFiles}
                    onToggleHidden={() => setShowHiddenFiles(!showHiddenFiles)}
                  />
                )}
              </div>
            </PanelShell>

            <PanelShell title={`Todo Queue (${todos.filter(t => !t.completed).length})`} tabs={['Active']}>
              <div className="stack" style={{ gap: '4px' }}>
                {/* Add new todo */}
                <div className="todo-input-row">
                  <input
                    type="text"
                    placeholder="Add a task..."
                    className="todo-input"
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        const input = e.target as HTMLInputElement;
                        if (input.value.trim()) {
                          useServerStore.getState().addTodo(activeProject, input.value.trim());
                          input.value = '';
                        }
                      }
                    }}
                  />
                  <button
                    className="todo-add-btn"
                    onClick={() => {
                      const input = document.querySelector('.todo-input') as HTMLInputElement;
                      if (input?.value.trim()) {
                        useServerStore.getState().addTodo(activeProject, input.value.trim());
                        input.value = '';
                      }
                    }}
                  >
                    +
                  </button>
                </div>
                
                {/* View all link */}
                {todos.length > 5 && (
                  <button
                    className="todo-view-all"
                    onClick={() => {
                      const nextTab: MainTab = {
                        id: `todos-${activeProject}`,
                        title: 'All Todos',
                        kind: 'todos',
                        payload: { projectId: activeProject },
                      };
                      setTabs((prev) => addOrReplaceTab(prev, nextTab));
                      setActiveTab(nextTab);
                    }}
                  >
                    View all {todos.length} tasks →
                  </button>
                )}
                
                {/* Todo list */}
                {todos.length === 0 ? (
                  <div className="empty-state" style={{ fontSize: '11px' }}>No tasks yet.</div>
                ) : (
                  todos.slice(0, 8).map((todo) => (
                    <div key={todo.id} className="todo-item">
                      <input
                        type="checkbox"
                        checked={todo.completed}
                        onChange={() => {
                          useServerStore.getState().setTodoCompleted(activeProject, todo.id, !todo.completed);
                        }}
                      />
                      <span className={`todo-text ${todo.completed ? 'completed' : ''}`}>
                        {todo.title}
                      </span>
                      <button
                        className="todo-remove"
                        onClick={() => {
                          useServerStore.getState().removeTodo(activeProject, todo.id);
                        }}
                        title="Remove"
                      >
                        ×
                      </button>
                    </div>
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
              ) : activeTab.kind === 'todos' ? (
                <TodosDetail 
                  projectId={(activeTab.payload as TodosPayload)?.projectId || activeProject} 
                />
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
      {isMcpModalOpen && activeProjectInfo && (
        <McpConfigModal
          projectName={activeProjectInfo.name}
          tools={mcpTools}
          results={mcpResults}
          isConfiguring={isMcpConfiguring}
          onConfigureAll={async () => {
            setIsMcpConfiguring(true);
            const results = await configureMcpForAllTools(activeProjectInfo.id);
            setMcpResults(results);
            setIsMcpConfiguring(false);
          }}
          onConfigureOne={async (tool: string) => {
            setIsMcpConfiguring(true);
            const result = await configureMcpForTool(tool, activeProjectInfo.id);
            setMcpResults((prev) => [...prev, result]);
            setIsMcpConfiguring(false);
          }}
          onClose={() => setIsMcpModalOpen(false)}
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
  kind: 'home' | 'tool' | 'file' | 'build' | 'todos';
  payload?: ToolCall | FilePayload | BuildPayload | TodosPayload;
};

type TodosPayload = {
  projectId: string;
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

function TodosDetail({ projectId }: { projectId: string }) {
  const { todos, addTodo, setTodoCompleted, removeTodo, loadTodos } = useServerStore();
  const [newTodo, setNewTodo] = useState('');
  
  // Load todos for this project when viewing
  useEffect(() => {
    loadTodos(projectId);
  }, [projectId, loadTodos]);
  
  const activeCount = todos.filter(t => !t.completed).length;
  
  return (
    <div className="detail-card">
      <div className="detail-header">
        <div>
          <div className="detail-title">Todo List</div>
          <div className="detail-subtitle">{activeCount} active, {todos.length - activeCount} completed</div>
        </div>
      </div>
      
      <div className="detail-section">
        {/* Add new todo */}
        <div className="todo-input-row" style={{ marginBottom: '16px' }}>
          <input
            type="text"
            placeholder="Add a new task..."
            className="todo-input"
            value={newTodo}
            onChange={(e) => setNewTodo(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && newTodo.trim()) {
                addTodo(projectId, newTodo.trim());
                setNewTodo('');
              }
            }}
          />
          <button
            className="todo-add-btn"
            onClick={() => {
              if (newTodo.trim()) {
                addTodo(projectId, newTodo.trim());
                setNewTodo('');
              }
            }}
          >
            Add
          </button>
        </div>
        
        {/* Todo list */}
        <div className="todo-list-full">
          {todos.length === 0 ? (
            <div className="empty-state">No tasks yet. Add one above!</div>
          ) : (
            todos.map((todo) => (
              <div key={todo.id} className={`todo-item-full ${todo.completed ? 'completed' : ''}`}>
                <input
                  type="checkbox"
                  checked={todo.completed}
                  onChange={() => setTodoCompleted(projectId, todo.id, !todo.completed)}
                />
                <div className="todo-content">
                  <div className="todo-title">{todo.title}</div>
                  {todo.description && <div className="todo-desc">{todo.description}</div>}
                </div>
                <button
                  className="todo-remove"
                  onClick={() => removeTodo(projectId, todo.id)}
                  title="Remove task"
                >
                  ×
                </button>
              </div>
            ))
          )}
        </div>
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
  onNavigateUp,
  showParent = false,
  showHidden = false,
  onToggleHidden,
}: {
  rootPath: string;
  listings: Record<string, DirectoryListing>;
  expanded: Record<string, boolean>;
  onToggle: (entry: DirectoryEntry) => void;
  onAddToContext: (entry: DirectoryEntry) => void;
  onNavigateUp?: () => void;
  showParent?: boolean;
  showHidden?: boolean;
  onToggleHidden?: () => void;
}) {
  if (!rootPath) {
    return <div className="empty-state">No directory selected.</div>;
  }

  const rootListing = listings[rootPath];
  if (!rootListing) {
    return <div className="empty-state">Loading tree...</div>;
  }

  return (
    <div className="tree-view">
      {/* Current path display with up navigation */}
      <div className="tree-header">
        {showParent && rootListing.parent_path && (
          <button className="tree-nav-up" onClick={onNavigateUp}>
            <span className="tree-icon">←</span>
            <span className="tree-nav-label">Parent</span>
          </button>
        )}
        <div className="tree-current-path" title={rootPath}>
          {rootPath}
        </div>
        {onToggleHidden && (
          <button 
            className="tree-hidden-toggle" 
            onClick={onToggleHidden}
            title={showHidden ? "Hide hidden files" : "Show hidden files"}
          >
            {showHidden ? "Hide .files" : "Show .files"}
          </button>
        )}
      </div>
      
      {/* Tree nodes */}
      <div className="tree-nodes-scrollable">
        <TreeNodes
          path={rootPath}
          listings={listings}
          expanded={expanded}
          onToggle={onToggle}
          onAddToContext={onAddToContext}
          depth={0}
          showHidden={showHidden}
        />
      </div>
    </div>
  );
}

function TreeNodes({
  path,
  listings,
  expanded,
  onToggle,
  onAddToContext,
  depth,
  showHidden,
}: {
  path: string;
  listings: Record<string, DirectoryListing>;
  expanded: Record<string, boolean>;
  onToggle: (entry: DirectoryEntry) => void;
  onAddToContext: (entry: DirectoryEntry) => void;
  depth: number;
  showHidden: boolean;
}) {
  const listing = listings[path];
  if (!listing) return null;

  // Filter out hidden files/folders (starting with .) unless showHidden is true
  const visibleEntries = showHidden 
    ? listing.entries 
    : listing.entries.filter(entry => !entry.name.startsWith('.'));

  return (
    <>
      {visibleEntries.map((entry) => {
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
            {entry.is_dir && isExpanded && (
              <TreeNodes
                path={entry.path}
                listings={listings}
                expanded={expanded}
                onToggle={onToggle}
                onAddToContext={onAddToContext}
                depth={depth + 1}
                showHidden={showHidden}
              />
            )}
          </div>
        );
      })}
    </>
  );
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

function McpConfigModal({
  projectName,
  tools,
  results,
  isConfiguring,
  onConfigureAll,
  onConfigureOne,
  onClose,
}: {
  projectName: string;
  tools: McpToolInfo[];
  results: McpConfigResult[];
  isConfiguring: boolean;
  onConfigureAll: () => void;
  onConfigureOne: (tool: string) => void;
  onClose: () => void;
}) {
  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        onClick={(event) => event.stopPropagation()}
        style={{ maxWidth: 600 }}
      >
        <div className="modal-title">Configure MCP for {projectName}</div>
        <p className="text-sm text-gray-600 mb-4">
          Add this project as an MCP server to your AI tools.
        </p>

        <div className="space-y-3 mb-4">
          <button
            className="button primary w-full"
            onClick={onConfigureAll}
            disabled={isConfiguring}
          >
            {isConfiguring ? 'Configuring...' : 'Configure All AI Tools'}
          </button>
        </div>

        <div className="border-t pt-4">
          <div className="text-sm font-semibold mb-2">Individual Tools:</div>
          <div className="grid grid-cols-2 gap-2">
            {tools.map((tool) => {
              const result = results.find((r) => r.config_path?.includes(tool.tool.toLowerCase()));
              return (
                <button
                  key={tool.tool}
                  className={`button ${result?.success ? 'success' : result ? 'error' : ''}`}
                  onClick={() => onConfigureOne(tool.tool.toLowerCase())}
                  disabled={isConfiguring}
                  title={tool.config_path ?? undefined}
                >
                  {tool.name}
                  {result && (result.success ? ' ✓' : ' ✗')}
                </button>
              );
            })}
          </div>
        </div>

        {results.length > 0 && (
          <div className="border-t pt-4 mt-4">
            <div className="text-sm font-semibold mb-2">Results:</div>
            <div className="space-y-2 max-h-40 overflow-y-auto">
              {results.map((result, index) => (
                <div
                  key={index}
                  className={`text-sm p-2 rounded ${result.success ? 'bg-green-100' : 'bg-red-100'}`}
                >
                  <div className={result.success ? 'text-green-800' : 'text-red-800'}>
                    {result.success ? '✓' : '✗'} {result.message}
                  </div>
                  {result.config_path && (
                    <div className="text-xs text-gray-600 mt-1">{result.config_path}</div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        <div className="modal-actions mt-4">
          <button className="button" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
