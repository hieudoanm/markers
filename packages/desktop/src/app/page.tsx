'use client';

import { useCallback, useEffect, useRef, useState } from 'react';

type FileOutcome = {
  name: string;
  ok: boolean;
  error: string | null;
  tei_path: string | null;
  md_path: string | null;
};

type UploadOutcome = {
  name: string;
  ok: boolean;
  error: string | null;
  tei_xml: string | null;
  md: string | null;
};

declare global {
  interface Window {
    __TAURI__?: {
      core?: { invoke: (cmd: string, args?: unknown) => Promise<unknown> };
      webview?: {
        getCurrentWebview?: () => {
          onDragDropEvent?: (
            cb: (e: { payload: { type: string; paths?: string[] } }) => void
          ) => Promise<() => void>;
        };
      };
    };
  }
}

const invoke = (cmd: string, args?: unknown) =>
  window.__TAURI__!.core!.invoke(cmd, args);

const isTauri = () =>
  typeof window !== 'undefined' && !!window.__TAURI__?.core?.invoke;

export default function DesktopPage() {
  const [tauri, setTauri] = useState<boolean | null>(null);
  const [paths, setPaths] = useState<string[]>([]);
  const [outDir, setOutDir] = useState<string>('');
  const [busy, setBusy] = useState(false);
  const [results, setResults] = useState<FileOutcome[] | null>(null);
  const [uploads, setUploads] = useState<UploadOutcome[] | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInput = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- Tauri API only exists client-side
    setTauri(isTauri());
    // Real file paths only exist inside Tauri — HTML5 drops don't expose them.
    const wv = window.__TAURI__?.webview?.getCurrentWebview?.();
    let unlisten: (() => void) | undefined;
    wv?.onDragDropEvent?.((e) => {
      if (e.payload.type === 'over' || e.payload.type === 'enter') {
        setDragOver(true);
      } else if (e.payload.type === 'leave') {
        setDragOver(false);
      } else if (e.payload.type === 'drop' && e.payload.paths) {
        setDragOver(false);
        setPaths((prev) => [...prev, ...e.payload.paths!]);
        setResults(null);
        setUploads(null);
      }
    })?.then((u) => {
      unlisten = u;
    });
    return () => unlisten?.();
  }, []);

  const pickPdfs = useCallback(async () => {
    const picked = (await invoke('pick_pdfs')) as string[];
    if (picked.length) {
      setPaths((prev) => [...prev, ...picked]);
      setResults(null);
      setUploads(null);
    }
  }, []);

  const pickDir = useCallback(async () => {
    const dir = (await invoke('pick_output_dir')) as string | null;
    if (dir) setOutDir(dir);
  }, []);

  const convert = useCallback(async () => {
    if (!paths.length) return;
    setBusy(true);
    setError(null);
    try {
      const res = (await invoke('convert_pdfs', {
        paths,
        outDir: outDir || null
      })) as FileOutcome[];
      setResults(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }, [paths, outDir]);

  const onFiles = useCallback(async (files: FileList | null) => {
    if (!files?.length) return;
    setBusy(true);
    setError(null);
    try {
      const payload = await Promise.all(
        Array.from(files).map(async (f) => ({
          name: f.name,
          data: Array.from(new Uint8Array(await f.arrayBuffer()))
        }))
      );
      const res = (await invoke('convert_uploads', {
        files: payload
      })) as UploadOutcome[];
      setUploads(res);
      setResults(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
      if (fileInput.current) fileInput.current.value = '';
    }
  }, []);

  if (tauri === null) {
    return (
      <main className="text-base-content/60 mx-auto flex w-full max-w-3xl flex-1 flex-col items-center justify-center px-6 py-24">
        Loading…
      </main>
    );
  }

  if (!tauri) {
    return (
      <main className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center px-6 py-16">
        <div className="card border-base-300 bg-base-100 border shadow-sm">
          <div className="card-body items-center gap-3 py-16 text-center">
            <h1 className="text-2xl font-semibold">Desktop only</h1>
            <p className="text-base-content/60 max-w-md">
              PDF → TEI/Markdown conversion runs inside the markers desktop app
              (Tauri). This page is a browser view — download the desktop build
              to convert files locally.
            </p>
            <code className="bg-base-200 rounded px-3 py-1 font-mono text-sm">
              pnpm desktop
            </code>
          </div>
        </div>
      </main>
    );
  }

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-1 flex-col px-6 py-10">
      <h1 className="text-3xl font-semibold tracking-tight">
        Desktop converter
      </h1>
      <p className="text-base-content/60 mt-2">
        Drop PDFs anywhere in this window, or pick files. Each PDF is converted
        to TEI XML and Markdown via a local GROBID service — no upload, no
        cloud.
      </p>

      <div
        onDragOver={(e) => {
          e.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onDrop={(e) => {
          // HTML5 drop is a fallback for non-path payloads only.
          e.preventDefault();
          setDragOver(false);
          void onFiles(e.dataTransfer.files);
        }}
        className={`mt-8 flex flex-1 flex-col items-center justify-center rounded-2xl border-2 border-dashed px-6 py-10 text-center transition-colors ${
          dragOver
            ? 'border-primary bg-primary/5'
            : 'border-base-300 bg-base-100'
        }`}
      >
        <p className="text-lg font-medium">
          {dragOver ? 'Drop PDFs to add them' : 'Drag & drop PDFs here'}
        </p>
        <p className="text-base-content/50 mt-1 text-sm">
          or use the buttons below — files keep their real paths
        </p>
        <div className="mt-6 flex flex-wrap items-center justify-center gap-3">
          <button
            type="button"
            className="btn btn-primary"
            onClick={pickPdfs}
            disabled={busy}
          >
            Choose PDFs…
          </button>
          <button
            type="button"
            className="btn btn-outline"
            onClick={() => fileInput.current?.click()}
            disabled={busy}
          >
            Upload files…
          </button>
          <input
            ref={fileInput}
            type="file"
            accept="application/pdf,.pdf"
            multiple
            className="hidden"
            onChange={(e) => void onFiles(e.target.files)}
          />
        </div>
      </div>

      {paths.length > 0 && (
        <div className="card border-base-300 bg-base-100 mt-6 border">
          <div className="card-body gap-3">
            <div className="flex items-center justify-between">
              <h2 className="font-semibold">
                {paths.length} PDF{paths.length === 1 ? '' : 's'} queued
              </h2>
              <button
                type="button"
                className="btn btn-ghost btn-sm"
                onClick={() => setPaths([])}
                disabled={busy}
              >
                Clear
              </button>
            </div>
            <ul className="text-base-content/70 max-h-40 space-y-1 overflow-y-auto font-mono text-xs">
              {paths.map((p) => (
                <li key={p} className="truncate">
                  {p}
                </li>
              ))}
            </ul>
            <div className="flex flex-wrap items-center gap-3">
              <button
                type="button"
                className="btn btn-secondary"
                onClick={pickDir}
                disabled={busy}
              >
                {outDir ? 'Output folder ✓' : 'Choose output folder…'}
              </button>
              {outDir && (
                <span className="truncate font-mono text-xs">{outDir}</span>
              )}
              <button
                type="button"
                className="btn btn-primary ml-auto"
                onClick={convert}
                disabled={busy}
              >
                {busy ? 'Converting…' : 'Convert'}
              </button>
            </div>
          </div>
        </div>
      )}

      {error && (
        <div className="alert alert-error mt-6">
          <span>{error}</span>
        </div>
      )}

      {results && (
        <ul className="mt-6 space-y-2">
          {results.map((r) => (
            <li
              key={r.name}
              className={`card bg-base-100 border px-4 py-3 ${
                r.ok ? 'border-success/40' : 'border-error/40'
              }`}
            >
              <div className="flex items-center justify-between gap-4">
                <span className="truncate font-medium">{r.name}</span>
                {r.ok ? (
                  <span className="text-success shrink-0 text-sm">
                    converted
                  </span>
                ) : (
                  <span className="text-error shrink-0 text-sm">{r.error}</span>
                )}
              </div>
              {r.ok && (
                <div className="text-base-content/50 mt-1 truncate font-mono text-xs">
                  {r.md_path}
                </div>
              )}
            </li>
          ))}
        </ul>
      )}

      {uploads && (
        <ul className="mt-6 space-y-4">
          {uploads.map((u) => (
            <li
              key={u.name}
              className="card border-base-300 bg-base-100 border"
            >
              <div className="card-body gap-2">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold">{u.name}</h3>
                  {u.ok ? (
                    <button
                      type="button"
                      className="btn btn-ghost btn-xs"
                      onClick={() => navigator.clipboard.writeText(u.md ?? '')}
                    >
                      Copy Markdown
                    </button>
                  ) : (
                    <span className="text-error text-sm">{u.error}</span>
                  )}
                </div>
                {u.ok && u.md && (
                  <pre className="bg-base-200 max-h-64 overflow-auto rounded p-3 text-xs">
                    {u.md.slice(0, 4000)}
                    {u.md.length > 4000 ? '\n…' : ''}
                  </pre>
                )}
              </div>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
