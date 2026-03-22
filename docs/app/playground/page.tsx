'use client';

import { useState, useEffect } from 'react';
import Editor, { useMonaco } from '@monaco-editor/react';
import { createHighlighter } from 'shiki';
import { shikiToMonaco } from '@shikijs/monaco';
import Link from 'next/link';
// @ts-ignore
import laadleGrammar from '@/laadle.tmLanguage.json';

let wasmInitPromise: Promise<any> | null = null;
let runLaadleCode: ((source: string) => string) | null = null;

const THEME = 'one-dark-pro';

export default function MinimalPlayground() {
  const [code, setCode] = useState<string>('laadle name hai "LaadleLang"\nbol "Welcome to " + name\n');
  const [output, setOutput] = useState<string>('');
  const [isWasmLoading, setIsWasmLoading] = useState(true);
  const [isEditorReady, setIsEditorReady] = useState(false);
  const monaco = useMonaco();

  useEffect(() => {
    async function loadWasm() {
      try {
        // @ts-ignore
        const wasmModule = await import('../../public/wasm/laadlelang.js');
        if (!wasmInitPromise) {
          wasmInitPromise = wasmModule.default({ module_or_path: '/wasm/laadlelang_bg.wasm?t=' + Date.now() });
        }
        await wasmInitPromise;
        runLaadleCode = wasmModule.run_laadle_code;
        setIsWasmLoading(false);
      } catch (e) {
        setOutput('Error: Failed to load WebAssembly VM.');
      }
    }
    loadWasm();
  }, []);

  useEffect(() => {
    if (!monaco) return;
    monaco.languages.register({ id: 'laadle' });

    async function mountShiki() {
      try {
        const highlighter = await createHighlighter({
          themes: [THEME],
          langs: [laadleGrammar as any],
        });
        shikiToMonaco(highlighter, monaco);
        setIsEditorReady(true);
      } catch (err) {}
    }
    mountShiki();
  }, [monaco]);

  const handleRun = () => {
    if (!runLaadleCode) return;
    setOutput('Compiling...\n');
    try {
      setTimeout(() => {
        if (!runLaadleCode) return;
        const res = runLaadleCode(code);
        setOutput(res || 'Execution finished (no output).');
      }, 50);
    } catch (e) {
      setOutput(`Error: ${e}`);
    }
  };

  return (
    <div className="flex flex-col h-screen overflow-hidden bg-[#282c34] font-sans">
      
      {/* Header */}
      <div className="flex items-center px-6 py-3 border-b border-[#181a1f] bg-[#21252b] shadow-sm relative z-10 w-full">
        
        {/* Left Side: Back Link & Title */}
        <div className="flex items-center gap-4">
          <Link href="/" className="text-[#abb2bf] hover:text-white transition-colors text-sm font-semibold flex items-center gap-2">
             <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 19l-7-7m0 0l7-7m-7 7h18" /></svg>
             <span className="hidden sm:inline">Back</span>
          </Link>
          <div className="h-4 w-px bg-[#3e4451]" />
          <h1 className="text-[#abb2bf] font-bold tracking-widest uppercase text-xs select-none">LaadleLang Playground</h1>
        </div>

        {/* Right Side: Run Button */}
        <button
          onClick={handleRun}
          disabled={isWasmLoading}
          className="flex items-center gap-2 px-5 py-2 bg-[#98c379] hover:bg-[#7cb653] active:bg-[#6a9e45] disabled:opacity-50 disabled:cursor-not-allowed text-[#282c34] text-[13.5px] font-bold rounded shadow-md transition-all ml-auto focus:outline-none focus:ring-2 focus:ring-[#98c379]/50"
        >
          {isWasmLoading ? (
             <svg className="animate-spin w-4 h-4 text-[#282c34]" fill="none" viewBox="0 0 24 24"><circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle><path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
          ) : (
             <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
          )}
          {isWasmLoading ? 'Waking Runtime...' : 'Run'}
        </button>
      </div>

      {/* 50/50 Editor Split */}
      <div className={`flex flex-1 flex-col md:flex-row overflow-hidden transition-opacity duration-300 ${isEditorReady ? 'opacity-100' : 'opacity-0'}`}>
        
        {/* Left: Input Editor */}
        <div 
          className="flex-[1] h-full border-r border-[#181a1f] relative group"
          onKeyDown={(e) => e.stopPropagation()}
        >
          <Editor
            height="100%"
            language="laadle"
            theme={THEME}
            value={code}
            onChange={(val) => setCode(val || '')}
            options={{
              minimap: { enabled: false },
              fontSize: 15,
              fontFamily: "'Fira Code', 'JetBrains Mono', 'Consolas', monospace",
              padding: { top: 24, bottom: 24 },
              scrollBeyondLastLine: false,
              wordWrap: 'on',
              cursorBlinking: 'smooth',
            }}
            loading={<div className="p-6 text-[#5c6370] font-mono text-sm animate-pulse">Initializing syntax highlighters...</div>}
          />
        </div>

        {/* Right: Output Editor */}
        <div 
          className="flex-[1] h-full bg-[#282c34] relative group"
          onKeyDown={(e) => e.stopPropagation()}
        >
           {/* Floating Clear Button - Only shows when there's output and user hovers right pane */}
          <div className={`absolute top-4 right-6 z-10 transition-opacity duration-200 ${output ? 'opacity-0 group-hover:opacity-100' : 'opacity-0 pointer-events-none'}`}>
             <button onClick={() => setOutput('')} className="bg-[#21252b] hover:bg-[#3e4451] text-[#abb2bf] hover:text-white px-3 py-1.5 text-xs font-semibold rounded shadow-sm transition-colors border border-[#181a1f]">
               Clear Output
             </button>
          </div>

          <Editor
            height="100%"
            language="plaintext"
            theme={THEME}
            value={output}
            options={{
              readOnly: true,
              minimap: { enabled: false },
              fontSize: 15,
              fontFamily: "'Fira Code', 'JetBrains Mono', 'Consolas', monospace",
              padding: { top: 24, bottom: 24 },
              scrollBeyondLastLine: false,
              wordWrap: 'on',
              lineNumbers: 'off',
              renderLineHighlight: 'none',
              hideCursorInOverviewRuler: true
            }}
            loading={null}
          />
        </div>

      </div>
    </div>
  );
}
