import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { X, Search, Filter, FileText, Terminal, Wrench, FolderSearch, Type } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Label } from "@/components/ui/label";
import type { SessionSearchQuery } from "@/types";

interface SessionSearchBarProps {
  onSearch: (query: SessionSearchQuery) => void;
  onClose: () => void;
}

export function SessionSearchBar({ onSearch, onClose }: SessionSearchBarProps) {
  const { t } = useTranslation();
  const [keywordQuery, setKeywordQuery] = useState("");
  const [filesQuery, setFilesQuery] = useState("");
  const [commandsQuery, setCommandsQuery] = useState("");
  const [toolsQuery, setToolsQuery] = useState("");
  const [projectQuery, setProjectQuery] = useState("");
  const [isDirty, setIsDirty] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Focus search input on mount
  useEffect(() => {
    searchInputRef.current?.focus();
  }, []);

  // Debounced search
  useEffect(() => {
    const timer = setTimeout(() => {
      if (keywordQuery || filesQuery || commandsQuery || toolsQuery || projectQuery) {
        setIsDirty(true);
      }
      const query: SessionSearchQuery = {
        keyword: keywordQuery.trim() || undefined,
        files: filesQuery.trim() || undefined,
        commands: commandsQuery.trim() || undefined,
        tools: toolsQuery.trim() || undefined,
        project: projectQuery.trim() || undefined,
      };
      onSearch(query);
    }, 300);

    return () => clearTimeout(timer);
  }, [keywordQuery, filesQuery, commandsQuery, toolsQuery, projectQuery, onSearch]);

  const handleClear = () => {
    setKeywordQuery("");
    setFilesQuery("");
    setCommandsQuery("");
    setToolsQuery("");
    setProjectQuery("");
    setIsDirty(false);
    onClose();
  };

  const handleReset = () => {
    setKeywordQuery("");
    setFilesQuery("");
    setCommandsQuery("");
    setToolsQuery("");
    setProjectQuery("");
    setIsDirty(false);
    onSearch({});
    searchInputRef.current?.focus();
  };

  const activeFilters = [
    { key: "keyword", value: keywordQuery, icon: Type },
    { key: "project", value: projectQuery, icon: FolderSearch },
    { key: "files", value: filesQuery, icon: FileText },
    { key: "commands", value: commandsQuery, icon: Terminal },
    { key: "tools", value: toolsQuery, icon: Wrench },
  ].filter(f => f.value);

  return (
    <div className="border-b bg-muted/30 p-3 space-y-3">
      {/* Main search row */}
      <div className="flex items-center gap-2">
        <div className="flex-1 relative">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <Input
            ref={searchInputRef}
            value={keywordQuery}
            onChange={(e) => setKeywordQuery(e.target.value)}
            placeholder={t("sessionManager.searchKeyword")}
            className="h-9 pl-9 pr-8 text-sm"
          />
          {keywordQuery && (
            <Button
              variant="ghost"
              size="icon"
              className="absolute right-1 top-1/2 -translate-y-1/2 size-7"
              onClick={() => setKeywordQuery("")}
            >
              <X className="size-3.5" />
            </Button>
          )}
        </div>

        <Popover>
          <PopoverTrigger asChild>
            <Button variant="outline" size="sm" className="gap-1.5">
              <Filter className="size-3.5" />
              {t("sessionManager.filters")}
              {activeFilters.length > 0 && (
                <Badge variant="secondary" className="ml-1 h-5 text-xs">
                  {activeFilters.length}
                </Badge>
              )}
            </Button>
          </PopoverTrigger>
          <PopoverContent 
            className="w-96 space-y-4" 
            align="start" 
            side="bottom"
            sideOffset={8}
            alignOffset={-100}
          >
            <div className="flex items-center justify-between">
              <h4 className="text-sm font-medium">
                {t("sessionManager.searchFilters")}
              </h4>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleReset}
                disabled={!isDirty}
              >
                {t("sessionManager.reset")}
              </Button>
            </div>

            {/* Keyword filter */}
            <div className="space-y-2">
              <Label className="text-xs flex items-center gap-1.5">
                <Type className="size-3.5" />
                {t("sessionManager.keyword")}
              </Label>
              <Input
                value={keywordQuery}
                onChange={(e) => setKeywordQuery(e.target.value)}
                placeholder={t("sessionManager.searchKeywordPlaceholder")}
                className="h-8 text-sm"
              />
            </div>

            {/* Project directory filter */}
            <div className="space-y-2">
              <Label className="text-xs flex items-center gap-1.5">
                <FolderSearch className="size-3.5" />
                {t("sessionManager.projectDirectory")}
              </Label>
              <Input
                value={projectQuery}
                onChange={(e) => setProjectQuery(e.target.value)}
                placeholder={t("sessionManager.searchProjectPlaceholder")}
                className="h-8 text-sm"
              />
            </div>

            {/* Files filter */}
            <div className="space-y-2">
              <Label className="text-xs flex items-center gap-1.5">
                <FileText className="size-3.5" />
                {t("sessionManager.filesModified")}
              </Label>
              <Input
                value={filesQuery}
                onChange={(e) => setFilesQuery(e.target.value)}
                placeholder={t("sessionManager.searchFilesPlaceholder")}
                className="h-8 text-sm"
              />
            </div>

            {/* Commands filter */}
            <div className="space-y-2">
              <Label className="text-xs flex items-center gap-1.5">
                <Terminal className="size-3.5" />
                {t("sessionManager.commandsExecuted")}
              </Label>
              <Input
                value={commandsQuery}
                onChange={(e) => setCommandsQuery(e.target.value)}
                placeholder={t("sessionManager.searchCommandsPlaceholder")}
                className="h-8 text-sm"
              />
            </div>

            {/* Tools filter */}
            <div className="space-y-2">
              <Label className="text-xs flex items-center gap-1.5">
                <Wrench className="size-3.5" />
                {t("sessionManager.toolsUsed")}
              </Label>
              <Input
                value={toolsQuery}
                onChange={(e) => setToolsQuery(e.target.value)}
                placeholder={t("sessionManager.searchToolsPlaceholder")}
                className="h-8 text-sm"
              />
              <p className="text-xs text-muted-foreground">
                {t("sessionManager.toolsHint")}
              </p>
            </div>

            {isDirty && (
              <div className="pt-2 border-t">
                <Button
                  variant="outline"
                  size="sm"
                  className="w-full"
                  onClick={handleReset}
                >
                  {t("sessionManager.clearFilters")}
                </Button>
              </div>
            )}
          </PopoverContent>
        </Popover>

        <Button variant="ghost" size="icon" className="size-9" onClick={handleClear}>
          <X className="size-4" />
        </Button>
      </div>

      {/* Active filters display */}
      {activeFilters.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {activeFilters.map((filter) => {
            const Icon = filter.icon;
            return (
              <Badge key={filter.key} variant="secondary" className="gap-1.5">
                <Icon className="size-3" />
                {filter.value}
                <button
                  type="button"
                  onClick={() => {
                    if (filter.key === "keyword") setKeywordQuery("");
                    else if (filter.key === "project") setProjectQuery("");
                    else if (filter.key === "files") setFilesQuery("");
                    else if (filter.key === "commands") setCommandsQuery("");
                    else if (filter.key === "tools") setToolsQuery("");
                  }}
                  className="ml-1 hover:text-foreground"
                >
                  <X className="size-3" />
                </button>
              </Badge>
            );
          })}
        </div>
      )}
    </div>
  );
}
