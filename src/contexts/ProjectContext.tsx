import React, { createContext, useContext, useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useServerStore } from '../stores/serverStore';
import type { ProjectInfo } from '../types';

interface ProjectContextType {
  currentProject: ProjectInfo | null;
  currentProjectId: string;
  setCurrentProjectId: (id: string) => void;
  isLoading: boolean;
}

const ProjectContext = createContext<ProjectContextType | null>(null);

export function ProjectProvider({ children }: { children: React.ReactNode }) {
  const { projects, loadTodos, loadContextNotes, loadBuildCommands, resetProjectData, setCurrentProject } = useServerStore();
  const [currentProjectId, setCurrentProjectIdState] = useState<string>('default');
  const [isLoading, setIsLoading] = useState(false);

  // Get current project info
  const currentProject = projects.find(p => p.id === currentProjectId) ?? null;

  // Centralized project switching
  const setCurrentProjectId = useCallback(async (id: string) => {
    if (id === currentProjectId) return;
    
    setIsLoading(true);
    console.log('[ProjectContext] Switching to project:', id);
    
    // Log to file
    try {
      await invoke('debug_log_cmd', { msg: `[ProjectContext] Switching from ${currentProjectId} to ${id}` });
    } catch { /* ignore */ }
    
    // 1. Clear all project-specific caches first
    resetProjectData();
    
    // 2. Update the current project ID
    setCurrentProjectIdState(id);
    
    // 3. Persist to localStorage
    localStorage.setItem('aiharness_last_project', id);
    
    setIsLoading(false);
  }, [currentProjectId, resetProjectData]);

  // Load initial project from localStorage
  useEffect(() => {
    const saved = localStorage.getItem('aiharness_last_project');
    if (saved && projects.some(p => p.id === saved)) {
      setCurrentProjectIdState(saved);
    } else if (projects.length > 0 && !projects.some(p => p.id === currentProjectId)) {
      // If current project doesn't exist, switch to first available
      setCurrentProjectIdState(projects[0].id);
    }
  }, [projects, currentProjectId]);

  // Auto-load data when project changes
  useEffect(() => {
    if (!currentProjectId) return;
    
    console.log('[ProjectContext] Auto-loading data for:', currentProjectId);
    
    // Update server-side current project for event filtering
    setCurrentProject(currentProjectId);
    
    // Load all project data
    loadTodos(currentProjectId);
    loadContextNotes(currentProjectId);
    loadBuildCommands(currentProjectId);
    
  }, [currentProjectId, loadTodos, loadContextNotes, loadBuildCommands, setCurrentProject]);

  return (
    <ProjectContext.Provider value={{
      currentProject,
      currentProjectId,
      setCurrentProjectId,
      isLoading
    }}>
      {children}
    </ProjectContext.Provider>
  );
}

export function useCurrentProject() {
  const context = useContext(ProjectContext);
  if (!context) {
    throw new Error('useCurrentProject must be used within ProjectProvider');
  }
  return context;
}
