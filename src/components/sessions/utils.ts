import { SessionMeta } from "@/types";

export const getSessionKey = (session: SessionMeta) =>
  `${session.providerId}:${session.sessionId}:${session.sourcePath ?? ""}`;

export const getBaseName = (value?: string | null) => {
  if (!value) return "";
  const trimmed = value.trim();
  if (!trimmed) return "";
  const normalized = trimmed.replace(/[\\/]+$/, "");
  const parts = normalized.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] || trimmed;
};

export const formatTimestamp = (value?: number) => {
  if (!value) return "";
  return new Date(value).toLocaleString();
};

export const formatRelativeTime = (
  value: number | undefined,
  t: (key: string, options?: Record<string, unknown>) => string,
) => {
  if (!value) return "";
  const now = Date.now();
  const diff = now - value;
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return t("sessionManager.justNow");
  if (minutes < 60) return t("sessionManager.minutesAgo", { count: minutes });
  if (hours < 24) return t("sessionManager.hoursAgo", { count: hours });
  if (days < 7) return t("sessionManager.daysAgo", { count: days });
  return new Date(value).toLocaleDateString();
};

/**
 * Group sessions by date for timeline view
 */
export interface GroupedSessions {
  label: string;
  date: Date;
  sessions: SessionMeta[];
}

export const groupSessionsByDate = (
  sessions: SessionMeta[],
  t: (key: string) => string,
): GroupedSessions[] => {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);
  const lastWeek = new Date(today);
  lastWeek.setDate(lastWeek.getDate() - 7);

  const groups = new Map<string, GroupedSessions>();

  for (const session of sessions) {
    const ts = session.lastActiveAt ?? session.createdAt ?? 0;
    if (!ts) continue;

    const sessionDate = new Date(ts);
    const sessionDay = new Date(
      sessionDate.getFullYear(),
      sessionDate.getMonth(),
      sessionDate.getDate(),
    );

    let label: string;

    if (sessionDay.getTime() >= today.getTime()) {
      label = t("sessionManager.today");
    } else if (sessionDay.getTime() >= yesterday.getTime()) {
      label = t("sessionManager.yesterday");
    } else if (sessionDay.getTime() >= lastWeek.getTime()) {
      // This week - use day name
      label = sessionDate.toLocaleDateString(undefined, { weekday: "long" });
    } else {
      // Older - use date format
      label = sessionDate.toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
      });
    }

    if (!groups.has(label)) {
      groups.set(label, {
        label,
        date: sessionDay,
        sessions: [],
      });
    }
    groups.get(label)!.sessions.push(session);
  }

  // Sort groups by date (newest first)
  const result = Array.from(groups.values()).sort((a, b) => {
    return b.date.getTime() - a.date.getTime();
  });

  // Sort sessions within each group by timestamp
  for (const group of result) {
    group.sessions.sort((a, b) => {
      const aTs = a.lastActiveAt ?? a.createdAt ?? 0;
      const bTs = b.lastActiveAt ?? b.createdAt ?? 0;
      return bTs - aTs;
    });
  }

  return result;
};

export const getProviderLabel = (
  providerId: string,
  t: (key: string) => string,
) => {
  const key = `apps.${providerId}`;
  const translated = t(key);
  return translated === key ? providerId : translated;
};

// 根据 providerId 获取对应的图标名称
export const getProviderIconName = (providerId: string) => {
  if (providerId === "codex") return "openai";
  if (providerId === "claude") return "claude";
  if (providerId === "opencode") return "opencode";
  if (providerId === "openclaw") return "openclaw";
  return providerId;
};

export const getRoleTone = (role: string) => {
  const normalized = role.toLowerCase();
  if (normalized === "assistant") return "text-blue-500";
  if (normalized === "user") return "text-emerald-500";
  if (normalized === "system") return "text-amber-500";
  if (normalized === "tool") return "text-purple-500";
  return "text-muted-foreground";
};

export const getRoleLabel = (role: string, t: (key: string) => string) => {
  const normalized = role.toLowerCase();
  if (normalized === "assistant") return "AI";
  if (normalized === "user") return t("sessionManager.roleUser");
  if (normalized === "system") return t("sessionManager.roleSystem");
  if (normalized === "tool") return t("sessionManager.roleTool");
  return role;
};

export const formatSessionTitle = (session: SessionMeta) => {
  return (
    session.title ||
    getBaseName(session.projectDir) ||
    session.sessionId.slice(0, 8)
  );
};
