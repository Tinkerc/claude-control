import { Clock, MessageSquare, FileText, Terminal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { SessionStats } from "@/types";

interface SessionStatsBadgeProps {
  stats?: SessionStats;
}

export function SessionStatsBadge({ stats }: SessionStatsBadgeProps) {
  const { t } = useTranslation();

  if (!stats) return null;

  const hasStats =
    stats.durationMinutes ||
    stats.messageCount ||
    stats.filesModified?.length ||
    stats.commandsExecuted?.length;

  if (!hasStats) return null;

  return (
    <div className="flex flex-wrap items-center gap-1 mt-1.5">
      {stats.durationMinutes !== undefined && stats.durationMinutes > 0 && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Badge
              variant="outline"
              className="text-[10px] h-5 px-1.5 gap-1 bg-blue-50 text-blue-700 border-blue-200"
            >
              <Clock className="size-3" />
              <span>{formatDuration(stats.durationMinutes)}</span>
            </Badge>
          </TooltipTrigger>
          <TooltipContent>
            {t("sessionManager.sessionDuration")}
          </TooltipContent>
        </Tooltip>
      )}

      {stats.messageCount !== undefined && stats.messageCount > 0 && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Badge
              variant="outline"
              className="text-[10px] h-5 px-1.5 gap-1 bg-purple-50 text-purple-700 border-purple-200"
            >
              <MessageSquare className="size-3" />
              <span>{stats.messageCount}</span>
            </Badge>
          </TooltipTrigger>
          <TooltipContent>
            {t("sessionManager.messageCount", {
              count: stats.messageCount,
            })}
          </TooltipContent>
        </Tooltip>
      )}

      {stats.filesModified && stats.filesModified.length > 0 && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Badge
              variant="outline"
              className="text-[10px] h-5 px-1.5 gap-1 bg-emerald-50 text-emerald-700 border-emerald-200"
            >
              <FileText className="size-3" />
              <span>{stats.filesModified.length}</span>
            </Badge>
          </TooltipTrigger>
          <TooltipContent>
            {t("sessionManager.filesModified", {
              count: stats.filesModified.length,
            })}
          </TooltipContent>
        </Tooltip>
      )}

      {stats.commandsExecuted && stats.commandsExecuted.length > 0 && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Badge
              variant="outline"
              className="text-[10px] h-5 px-1.5 gap-1 bg-amber-50 text-amber-700 border-amber-200"
            >
              <Terminal className="size-3" />
              <span>{stats.commandsExecuted.length}</span>
            </Badge>
          </TooltipTrigger>
          <TooltipContent>
            {t("sessionManager.commandsExecuted", {
              count: stats.commandsExecuted.length,
            })}
          </TooltipContent>
        </Tooltip>
      )}
    </div>
  );
}

function formatDuration(minutes: number): string {
  if (minutes < 60) {
    return `${minutes}m`;
  }
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (mins === 0) {
    return `${hours}h`;
  }
  return `${hours}h${mins}m`;
}
