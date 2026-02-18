import { useState, useEffect, useRef, useCallback } from 'react';
import { api } from '../services/api';

export interface LogEntry {
  timestamp: number;
  level: 'log' | 'warn' | 'error' | 'info' | 'event';
  source: 'frontend' | 'backend';
  message: string;
  repeatCount?: number;
}

const MAX_ENTRIES = 500;

/** Singleton log buffer shared across hook instances */
const logBuffer: LogEntry[] = [];
let consolePatched = false;
const listeners = new Set<() => void>();

function pushEntry(entry: LogEntry) {
  const last = logBuffer[logBuffer.length - 1];
  if (last && last.message === entry.message && last.level === entry.level && last.source === entry.source) {
    last.repeatCount = (last.repeatCount || 1) + 1;
    last.timestamp = entry.timestamp;
  } else {
    logBuffer.push(entry);
    if (logBuffer.length > MAX_ENTRIES) logBuffer.splice(0, logBuffer.length - MAX_ENTRIES);
  }
  listeners.forEach(fn => fn());
}

function patchConsole() {
  if (consolePatched) return;
  consolePatched = true;

  const levels = ['log', 'info', 'warn', 'error'] as const;
  for (const level of levels) {
    const original = console[level];
    console[level] = (...args: any[]) => {
      original.apply(console, args);
      const message = args.map(a =>
        typeof a === 'string' ? a : (a instanceof Error ? a.message : JSON.stringify(a, null, 0))
      ).join(' ');
      pushEntry({ timestamp: Date.now(), level, source: 'frontend', message });
    };
  }
}

export function useInternalLogs() {
  const [, setTick] = useState(0);
  const pollingRef = useRef(false);

  // Subscribe to buffer changes
  useEffect(() => {
    const handler = () => setTick(t => t + 1);
    listeners.add(handler);
    return () => { listeners.delete(handler); };
  }, []);

  // Patch console on first mount
  useEffect(() => { patchConsole(); }, []);

  // Poll backend events
  useEffect(() => {
    if (pollingRef.current) return;
    pollingRef.current = true;

    const poll = async () => {
      try {
        const result = await api.pollEvents(50);
        const events = result?.events;
        if (Array.isArray(events)) {
          for (const ev of events) {
            const isLogEntry = ev?.kind === 'log';
            const msg = typeof ev === 'string' ? ev
              : (ev?.message || ev?.type || JSON.stringify(ev));
            pushEntry({
              timestamp: Date.now(),
              level: isLogEntry ? 'info' : 'event',
              source: 'backend',
              message: msg,
            });
          }
        }
      } catch {
        // silently skip if backend unavailable
      }
    };

    poll();
    const interval = setInterval(poll, 2000);
    return () => { clearInterval(interval); pollingRef.current = false; };
  }, []);

  const clear = useCallback(() => {
    logBuffer.length = 0;
    setTick(t => t + 1);
  }, []);

  const copyToClipboard = useCallback(async () => {
    const text = logBuffer.map(e => {
      const ts = new Date(e.timestamp).toISOString().slice(11, 23);
      const repeat = (e.repeatCount || 0) > 1 ? ` (x${e.repeatCount})` : '';
      return `[${ts}] [${e.source}/${e.level}] ${e.message}${repeat}`;
    }).join('\n');
    await navigator.clipboard.writeText(text);
  }, []);

  return { entries: logBuffer, clear, copyToClipboard };
}
