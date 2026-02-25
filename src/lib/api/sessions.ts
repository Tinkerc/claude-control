import { invoke } from "@tauri-apps/api/core";
import type {
  SessionMessage,
  SessionMeta,
  ProjectStats,
  WorkflowPatterns,
  ContentAnalysis,
  SimilarSession,
  AllInsights,
} from "@/types";

export interface SessionSearchQuery {
  keyword?: string;
  files?: string;
  commands?: string;
  tools?: string;
  project?: string;
  provider?: string;
  startTime?: number;
  endTime?: number;
}

export interface SessionSearchResult {
  sessions: SessionMeta[];
  total: number;
  matchedFiles?: string[];
  matchedCommands?: string[];
  matchedTools?: string[];
}

export const sessionsApi = {
  async list(): Promise<SessionMeta[]> {
    return await invoke("list_sessions");
  },

  async getMessages(
    providerId: string,
    sourcePath: string,
  ): Promise<SessionMessage[]> {
    return await invoke("get_session_messages", { providerId, sourcePath });
  },

  async launchTerminal(options: {
    command: string;
    cwd?: string | null;
    customConfig?: string | null;
  }): Promise<boolean> {
    const { command, cwd, customConfig } = options;
    return await invoke("launch_session_terminal", {
      command,
      cwd,
      customConfig,
    });
  },

  async search(query: SessionSearchQuery): Promise<SessionSearchResult> {
    return await invoke("search_sessions", { query });
  },

  // Insights API methods
  async getProjectStats(projectDir?: string): Promise<ProjectStats[]> {
    return await invoke("get_project_stats", { projectDir });
  },

  async getWorkflowPatterns(projectDir?: string): Promise<WorkflowPatterns> {
    return await invoke("get_workflow_patterns", { projectDir });
  },

  async getContentAnalysis(projectDir?: string): Promise<ContentAnalysis> {
    return await invoke("get_content_analysis", { projectDir });
  },

  async findSimilarSessions(sessionId: string, limit?: number): Promise<SimilarSession[]> {
    return await invoke("find_similar_sessions", { sessionId, limit });
  },

  async getAllInsights(projectDir?: string): Promise<AllInsights> {
    return await invoke("get_all_insights", { projectDir });
  },
};
