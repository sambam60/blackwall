"use client";

import { useCallback, useRef, useState } from "react";

type CommandTab = {
  id: string;
  label: string;
  command: string;
};

export function CopyCommand({ commands }: { commands: CommandTab[] }) {
  const [activeId, setActiveId] = useState(commands[0]?.id ?? "");
  const [copied, setCopied] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const activeCommand = commands.find((item) => item.id === activeId) ?? commands[0];

  const handleCopy = useCallback(() => {
    if (!activeCommand) return;
    navigator.clipboard.writeText(activeCommand.command);
    setCopied(true);
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    timeoutRef.current = setTimeout(() => setCopied(false), 1500);
  }, [activeCommand]);

  if (!activeCommand) return null;

  return (
    <div className="flex flex-col gap-2 w-full sm:w-auto">
      <div className="flex flex-wrap items-center gap-1 text-xs font-mono">
        {commands.map((item) => {
          const isActive = item.id === activeCommand.id;
          return (
            <button
              key={item.id}
              onClick={() => {
                setActiveId(item.id);
                setCopied(false);
              }}
              className={`rounded px-2.5 py-1 transition-colors ${
                isActive
                  ? "bg-foreground text-background border border-foreground"
                  : "text-muted hover:text-foreground border border-border-light hover:bg-surface-tint"
              }`}
              aria-label={`Use ${item.label} install command`}
            >
              {item.label}
            </button>
          );
        })}
      </div>

      <div className="flex items-center gap-3 bg-card border border-border-light rounded-lg px-4 sm:px-5 py-3 text-sm font-mono overflow-x-auto">
        <span className="text-muted-darker shrink-0">$</span>
        <span className="text-foreground/80 select-all">{activeCommand.command}</span>
        <button
          onClick={handleCopy}
          className="text-muted hover:text-foreground transition-colors ml-1 shrink-0"
          aria-label="Copy"
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className={`transition-transform duration-150 ${copied ? "scale-110" : ""}`}
          >
            {copied ? (
              <polyline points="20 6 9 17 4 12" />
            ) : (
              <>
                <rect x="9" y="9" width="13" height="13" rx="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </>
            )}
          </svg>
        </button>
      </div>
    </div>
  );
}
