import { useCallback, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { sessionsApi, type SessionSearchQuery, type SessionSearchResult } from "@/lib/api/sessions";

interface UseBackendSessionSearchOptions {
  enabled?: boolean;
}

interface UseBackendSessionSearchResult {
  search: (query: SessionSearchQuery) => void;
  result: SessionSearchResult | null;
  isLoading: boolean;
  error: Error | null;
  reset: () => void;
}

/**
 * Backend-powered session search using the new search_sessions command
 * Searches across files modified, commands executed, and tools used
 */
export function useBackendSessionSearch(
  options: UseBackendSessionSearchOptions = {},
): UseBackendSessionSearchResult {
  const { enabled = true } = options;
  const [query, setQuery] = useState<SessionSearchQuery | null>(null);

  const { data, isLoading, error } = useQuery<SessionSearchResult | null>({
    queryKey: ["sessionSearch", query],
    queryFn: async () => {
      if (!query) return null;
      return await sessionsApi.search(query);
    },
    enabled: enabled && query !== null,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });

  const search = useCallback((newQuery: SessionSearchQuery) => {
    setQuery(newQuery);
  }, []);

  const reset = useCallback(() => {
    setQuery(null);
  }, []);

  return {
    search,
    result: data ?? null,
    isLoading,
    error: error as Error | null,
    reset,
  };
}
