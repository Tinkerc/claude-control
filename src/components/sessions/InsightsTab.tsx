import { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  BarChart3,
  TrendingUp,
  Tag,
  RefreshCw,
  Search,
  X,
} from "lucide-react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  useAllInsightsQuery,
  useProjectStatsQuery,
  useWorkflowPatternsQuery,
  useContentAnalysisQuery,
  useSimilarSessionsQuery,
} from "@/lib/query";
import { sessionsApi } from "@/lib/api";
import { extractErrorMessage } from "@/utils/errorUtils";

type ViewMode = "overview" | "projects" | "workflows" | "content";

export function InsightsTab() {
  const { t } = useTranslation();
  const [viewMode, setViewMode] = useState<ViewMode>("overview");
  const [selectedProject, setSelectedProject] = useState<string>("all");
  const [similarSessionModalOpen, setSimilarSessionModalOpen] = useState(false);
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);

  const projectFilter = selectedProject === "all" ? undefined : selectedProject;

  // Queries
  const {
    data: allInsights,
    isLoading: isLoadingAll,
    refetch: refetchAll,
  } = useAllInsightsQuery(projectFilter);

  const {
    data: projectStats,
    isLoading: isLoadingProjects,
    refetch: refetchProjects,
  } = useProjectStatsQuery(projectFilter);

  const {
    data: workflowPatterns,
    isLoading: isLoadingWorkflows,
    refetch: refetchWorkflows,
  } = useWorkflowPatternsQuery(projectFilter);

  const {
    data: contentAnalysis,
    isLoading: isLoadingContent,
    refetch: refetchContent,
  } = useContentAnalysisQuery(projectFilter);

  const {
    data: similarSessions,
    isLoading: isLoadingSimilar,
  } = useSimilarSessionsQuery(selectedSessionId ?? undefined, 5);

  const handleRefresh = async () => {
    try {
      await Promise.all([
        refetchAll(),
        refetchProjects(),
        refetchWorkflows(),
        refetchContent(),
      ]);
      toast.success(t("insights.refreshed", { defaultValue: "Insights refreshed" }));
    } catch (error) {
      toast.error(
        extractErrorMessage(error) ||
          t("common.error", { defaultValue: "Failed to refresh insights" }),
      );
    }
  };

  const isLoading = isLoadingAll || isLoadingProjects || isLoadingWorkflows || isLoadingContent;

  const uniqueProjects = projectStats?.map((p) => p.projectDir) ?? [];

  return (
    <div className="flex flex-col h-full gap-4">
      {/* Header */}
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-2">
          <BarChart3 className="size-5 text-muted-foreground" />
          <h2 className="text-lg font-semibold">
            {t("insights.title", { defaultValue: "Session Insights" })}
          </h2>
          {allInsights && (
            <Badge variant="secondary" className="text-xs">
              {allInsights.totalSessionsAnalyzed} {t("insights.sessions", { defaultValue: "sessions" })}
            </Badge>
          )}
        </div>
        <div className="flex items-center gap-2">
          {/* Project Filter */}
          {uniqueProjects.length > 0 && (
            <Select value={selectedProject} onValueChange={setSelectedProject}>
              <SelectTrigger className="w-[200px]">
                <SelectValue
                  placeholder={t("insights.allProjects", { defaultValue: "All Projects" })}
                />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">
                  {t("insights.allProjects", { defaultValue: "All Projects" })}
                </SelectItem>
                {uniqueProjects.map((project) => (
                  <SelectItem key={project} value={project}>
                    {project}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}

          <Button
            variant="outline"
            size="sm"
            className="gap-1.5"
            onClick={() => void handleRefresh()}
            disabled={isLoading}
          >
            <RefreshCw className={`size-3.5 ${isLoading ? "animate-spin" : ""}`} />
            <span className="hidden sm:inline">
              {t("common.refresh", { defaultValue: "Refresh" })}
            </span>
          </Button>
        </div>
      </div>

      {/* View Mode Tabs */}
      <div className="flex items-center gap-1 border-b">
        <Button
          variant={viewMode === "overview" ? "default" : "ghost"}
          size="sm"
          onClick={() => setViewMode("overview")}
          className="gap-1.5"
        >
          <BarChart3 className="size-3.5" />
          {t("insights.overview", { defaultValue: "Overview" })}
        </Button>
        <Button
          variant={viewMode === "projects" ? "default" : "ghost"}
          size="sm"
          onClick={() => setViewMode("projects")}
          className="gap-1.5"
        >
          <Tag className="size-3.5" />
          {t("insights.projects", { defaultValue: "Projects" })}
        </Button>
        <Button
          variant={viewMode === "workflows" ? "default" : "ghost"}
          size="sm"
          onClick={() => setViewMode("workflows")}
          className="gap-1.5"
        >
          <TrendingUp className="size-3.5" />
          {t("insights.workflows", { defaultValue: "Workflows" })}
        </Button>
        <Button
          variant={viewMode === "content" ? "default" : "ghost"}
          size="sm"
          onClick={() => setViewMode("content")}
          className="gap-1.5"
        >
          <Search className="size-3.5" />
          {t("insights.content", { defaultValue: "Content" })}
        </Button>
      </div>

      {/* Content Area */}
      <ScrollArea className="flex-1">
        <div className="p-4">
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <RefreshCw className="size-5 animate-spin text-muted-foreground" />
            </div>
          ) : !allInsights ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <BarChart3 className="size-8 text-muted-foreground/50 mb-2" />
              <p className="text-sm text-muted-foreground">
                {t("insights.noData", { defaultValue: "No session data available" })}
              </p>
            </div>
          ) : (
            <>
              {viewMode === "overview" && (
                <OverviewPanel insights={allInsights} />
              )}
              {viewMode === "projects" && (
                <ProjectStatsPanel stats={projectStats ?? []} />
              )}
              {viewMode === "workflows" && workflowPatterns && (
                <WorkflowPatternsPanel patterns={workflowPatterns} />
              )}
              {viewMode === "content" && contentAnalysis && (
                <ContentAnalysisPanel analysis={contentAnalysis} />
              )}
            </>
          )}
        </div>
      </ScrollArea>

      {/* Similar Sessions Modal */}
      {similarSessionModalOpen && selectedSessionId && (
        <SimilarSessionsModal
          sessionId={selectedSessionId}
          similarSessions={similarSessions ?? []}
          isLoading={isLoadingSimilar}
          onClose={() => {
            setSimilarSessionModalOpen(false);
            setSelectedSessionId(null);
          }}
        />
      )}
    </div>
  );
}

// ============================================================================
// Sub-components
// ============================================================================

interface OverviewPanelProps {
  insights: {
    projectStats: {
      projectDir: string;
      sessionCount: number;
      totalDurationMinutes: number;
      avgSessionDuration: number;
    }[];
    workflowPatterns: {
      toolUsageDistribution: { tool: string; usageCount: number }[];
      sessionLengthDistribution: { range: string; count: number }[];
      mostCommonSequences: string[];
    };
    contentAnalysis: {
      taskClassification: { category: string; count: number }[];
      concepts: { concept: string; mentions: number }[];
    };
    totalSessionsAnalyzed: number;
  };
}

function OverviewPanel({ insights }: OverviewPanelProps) {
  const { t } = useTranslation();

  const topProjects = insights.projectStats.slice(0, 5);
  const topTasks = insights.contentAnalysis.taskClassification
    .sort((a, b) => b.count - a.count)
    .slice(0, 5);
  const topConcepts = insights.contentAnalysis.concepts
    .sort((a, b) => b.mentions - a.mentions)
    .slice(0, 10);

  return (
    <div className="space-y-4">
      {/* Summary Stats */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold">{insights.totalSessionsAnalyzed}</div>
            <div className="text-xs text-muted-foreground">
              {t("insights.totalSessions", { defaultValue: "Total Sessions" })}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold">{insights.projectStats.length}</div>
            <div className="text-xs text-muted-foreground">
              {t("insights.totalProjects", { defaultValue: "Total Projects" })}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold">
              {Math.round(
                insights.projectStats.reduce(
                  (sum, p) => sum + p.avgSessionDuration,
                  0,
                ) / Math.max(insights.projectStats.length, 1),
              )}
              min
            </div>
            <div className="text-xs text-muted-foreground">
              {t("insights.avgSessionDuration", { defaultValue: "Avg Session Duration" })}
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Top Projects */}
        <Card>
          <CardHeader className="py-3 px-4">
            <CardTitle className="text-sm font-medium">
              {t("insights.topProjects", { defaultValue: "Top Projects by Sessions" })}
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-4">
            {topProjects.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                {t("insights.noProjects", { defaultValue: "No projects found" })}
              </p>
            ) : (
              <div className="space-y-2">
                {topProjects.map((project) => (
                  <div
                    key={project.projectDir}
                    className="flex items-center justify-between text-sm"
                  >
                    <span className="truncate flex-1">{project.projectDir}</span>
                    <Badge variant="secondary" className="ml-2">
                      {project.sessionCount}
                    </Badge>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Task Distribution */}
        <Card>
          <CardHeader className="py-3 px-4">
            <CardTitle className="text-sm font-medium">
              {t("insights.taskDistribution", { defaultValue: "Task Distribution" })}
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-4">
            {topTasks.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                {t("insights.noTasks", { defaultValue: "No tasks classified" })}
              </p>
            ) : (
              <div className="space-y-2">
                {topTasks.map((task) => (
                  <div
                    key={task.category}
                    className="flex items-center justify-between text-sm"
                  >
                    <span className="capitalize">{task.category}</span>
                    <Badge variant="secondary">{task.count}</Badge>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Top Concepts */}
        <Card className="md:col-span-2">
          <CardHeader className="py-3 px-4">
            <CardTitle className="text-sm font-medium">
              {t("insights.topConcepts", { defaultValue: "Top Technical Concepts" })}
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-4">
            {topConcepts.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                {t("insights.noConcepts", { defaultValue: "No concepts found" })}
              </p>
            ) : (
              <div className="flex flex-wrap gap-2">
                {topConcepts.map((concept) => (
                  <Badge key={concept.concept} variant="outline">
                    {concept.concept} ({concept.mentions})
                  </Badge>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

interface ProjectStatsPanelProps {
  stats: {
    projectDir: string;
    sessionCount: number;
    totalDurationMinutes: number;
    topCommands: { command: string; count: number }[];
    topFiles: { file: string; editCount: number }[];
    topTools: { tool: string; usageCount: number }[];
    avgSessionDuration: number;
  }[];
}

function ProjectStatsPanel({ stats }: ProjectStatsPanelProps) {
  const { t } = useTranslation();

  if (stats.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12 text-center">
        <Tag className="size-8 text-muted-foreground/50 mb-2" />
        <p className="text-sm text-muted-foreground">
          {t("insights.noProjectStats", { defaultValue: "No project statistics available" })}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {stats.map((project) => (
        <Card key={project.projectDir}>
          <CardHeader className="py-3 px-4">
            <div className="flex items-start justify-between">
              <CardTitle className="text-sm font-medium truncate flex-1">
                {project.projectDir}
              </CardTitle>
              <div className="flex items-center gap-2 ml-2">
                <Badge variant="secondary">
                  {project.sessionCount} {t("insights.sessions", { defaultValue: "sessions" })}
                </Badge>
                <Badge variant="outline">
                  {Math.round(project.totalDurationMinutes / 60)}h
                </Badge>
              </div>
            </div>
          </CardHeader>
          <CardContent className="px-4 pb-4">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {/* Top Commands */}
              {project.topCommands.length > 0 && (
                <div>
                  <h4 className="text-xs font-medium text-muted-foreground mb-2">
                    {t("insights.topCommands", { defaultValue: "Top Commands" })}
                  </h4>
                  <div className="space-y-1">
                    {project.topCommands.slice(0, 5).map((cmd) => (
                      <div
                        key={cmd.command}
                        className="flex items-center justify-between text-xs"
                      >
                        <code className="truncate flex-1">{cmd.command}</code>
                        <Badge variant="secondary" className="ml-1 text-xs">
                          {cmd.count}
                        </Badge>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Top Files */}
              {project.topFiles.length > 0 && (
                <div>
                  <h4 className="text-xs font-medium text-muted-foreground mb-2">
                    {t("insights.topFiles", { defaultValue: "Top Files" })}
                  </h4>
                  <div className="space-y-1">
                    {project.topFiles.slice(0, 5).map((file) => (
                      <div
                        key={file.file}
                        className="flex items-center justify-between text-xs"
                      >
                        <span className="truncate flex-1 font-mono">{file.file}</span>
                        <Badge variant="secondary" className="ml-1 text-xs">
                          {file.editCount}
                        </Badge>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Top Tools */}
              {project.topTools.length > 0 && (
                <div>
                  <h4 className="text-xs font-medium text-muted-foreground mb-2">
                    {t("insights.topTools", { defaultValue: "Top Tools" })}
                  </h4>
                  <div className="space-y-1">
                    {project.topTools.slice(0, 5).map((tool) => (
                      <div
                        key={tool.tool}
                        className="flex items-center justify-between text-xs"
                      >
                        <span className="truncate flex-1">{tool.tool}</span>
                        <Badge variant="secondary" className="ml-1 text-xs">
                          {tool.usageCount}
                        </Badge>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

interface WorkflowPatternsPanelProps {
  patterns: {
    commandSequences: { sequence: string; occurrenceCount: number; exampleSessionId: string }[];
    toolSequences: { pattern: string; occurrenceCount: number; exampleSessionId: string }[];
    toolUsageDistribution: { tool: string; usageCount: number }[];
    sessionLengthDistribution: { range: string; count: number }[];
    mostCommonSequences: string[];
  };
}

function WorkflowPatternsPanel({ patterns }: WorkflowPatternsPanelProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-4">
      {/* Tool Usage Distribution */}
      {patterns.toolUsageDistribution.length > 0 && (
        <Card>
          <CardHeader className="py-3 px-4">
            <CardTitle className="text-sm font-medium">
              {t("insights.toolUsageDistribution", { defaultValue: "Tool Usage Distribution" })}
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-4">
            <div className="flex flex-wrap gap-2">
              {patterns.toolUsageDistribution.map((tool) => (
                <Badge key={tool.tool} variant="outline">
                  {tool.tool} ({tool.usageCount})
                </Badge>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Session Length Distribution */}
      {patterns.sessionLengthDistribution.length > 0 && (
        <Card>
          <CardHeader className="py-3 px-4">
            <CardTitle className="text-sm font-medium">
              {t("insights.sessionLengthDistribution", { defaultValue: "Session Length Distribution" })}
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-4">
            <div className="space-y-2">
              {patterns.sessionLengthDistribution.map((dist) => (
                <div key={dist.range} className="flex items-center gap-2">
                  <span className="text-xs w-16">{dist.range}</span>
                  <div className="flex-1 h-4 bg-muted rounded overflow-hidden">
                    <div
                      className="h-full bg-primary"
                      style={{
                        width: `${(dist.count /
                          Math.max(...patterns.sessionLengthDistribution.map((d) => d.count))) *
                          100}%`,
                      }}
                    />
                  </div>
                  <Badge variant="secondary" className="text-xs">
                    {dist.count}
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Command Sequences */}
      {patterns.commandSequences.length > 0 && (
        <Card>
          <CardHeader className="py-3 px-4">
            <CardTitle className="text-sm font-medium">
              {t("insights.repeatedCommandSequences", { defaultValue: "Repeated Command Sequences" })}
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-4">
            <div className="space-y-2">
              {patterns.commandSequences.map((seq) => (
                <div key={seq.sequence} className="text-sm">
                  <code className="text-xs bg-muted px-2 py-1 rounded">
                    {seq.sequence}
                  </code>
                  <Badge variant="secondary" className="ml-2">
                    {seq.occurrenceCount}x
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

interface ContentAnalysisPanelProps {
  analysis: {
    taskClassification: { category: string; count: number }[];
    concepts: { concept: string; mentions: number }[];
    promptPatterns: { pattern: string; occurrenceCount: number; exampleUsage?: string }[];
    totalSessionsAnalyzed: number;
  };
}

function ContentAnalysisPanel({ analysis }: ContentAnalysisPanelProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-4">
      {/* Task Classification */}
      <Card>
        <CardHeader className="py-3 px-4">
          <CardTitle className="text-sm font-medium">
            {t("insights.taskClassification", { defaultValue: "Task Classification" })}
          </CardTitle>
        </CardHeader>
        <CardContent className="px-4 pb-4">
          {analysis.taskClassification.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t("insights.noTasks", { defaultValue: "No tasks classified" })}
            </p>
          ) : (
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
              {analysis.taskClassification.map((task) => (
                <div key={task.category} className="text-center p-2 border rounded">
                  <div className="text-lg font-bold">{task.count}</div>
                  <div className="text-xs text-muted-foreground capitalize">
                    {task.category.replace(/_/g, " ")}
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Technical Concepts */}
      {analysis.concepts.length > 0 && (
        <Card>
          <CardHeader className="py-3 px-4">
            <CardTitle className="text-sm font-medium">
              {t("insights.technicalConcepts", { defaultValue: "Technical Concepts" })}
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-4">
            <div className="flex flex-wrap gap-2">
              {analysis.concepts.map((concept) => (
                <Badge key={concept.concept} variant="outline">
                  {concept.concept} ({concept.mentions})
                </Badge>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Prompt Patterns */}
      {analysis.promptPatterns.length > 0 && (
        <Card>
          <CardHeader className="py-3 px-4">
            <CardTitle className="text-sm font-medium">
              {t("insights.promptPatterns", { defaultValue: "Common Prompt Patterns" })}
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-4">
            <div className="space-y-2">
              {analysis.promptPatterns.map((pattern) => (
                <div key={pattern.pattern} className="text-sm">
                  <code className="text-xs bg-muted px-2 py-1 rounded">
                    {pattern.pattern}
                  </code>
                  <Badge variant="secondary" className="ml-2">
                    {pattern.occurrenceCount}x
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

interface SimilarSessionsModalProps {
  sessionId: string;
  similarSessions: {
    sessionId: string;
    similarityScore: number;
    similarityReason: string;
  }[];
  isLoading: boolean;
  onClose: () => void;
}

function SimilarSessionsModal({
  sessionId,
  similarSessions,
  isLoading,
  onClose,
}: SimilarSessionsModalProps) {
  const { t } = useTranslation();

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
      <Card className="w-full max-w-lg max-h-[80vh] overflow-hidden">
        <CardHeader className="py-3 px-4 border-b">
          <div className="flex items-center justify-between">
            <CardTitle className="text-sm font-medium">
              {t("insights.similarSessions", { defaultValue: "Similar Sessions" })}
            </CardTitle>
            <Button variant="ghost" size="icon" className="size-7" onClick={onClose}>
              <X className="size-4" />
            </Button>
          </div>
        </CardHeader>
        <CardContent className="p-4">
          <ScrollArea className="h-[400px]">
            {isLoading ? (
              <div className="flex items-center justify-center py-12">
                <RefreshCw className="size-5 animate-spin text-muted-foreground" />
              </div>
            ) : similarSessions.length === 0 ? (
              <p className="text-sm text-muted-foreground text-center py-8">
                {t("insights.noSimilarSessions", { defaultValue: "No similar sessions found" })}
              </p>
            ) : (
              <div className="space-y-2">
                {similarSessions.map((session) => (
                  <div key={session.sessionId} className="p-3 border rounded">
                    <div className="flex items-center justify-between mb-1">
                      <code className="text-xs">{session.sessionId}</code>
                      <Badge variant="secondary">
                        {Math.round(session.similarityScore * 100)}%
                      </Badge>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {session.similarityReason}
                    </p>
                  </div>
                ))}
              </div>
            )}
          </ScrollArea>
        </CardContent>
      </Card>
    </div>
  );
}
