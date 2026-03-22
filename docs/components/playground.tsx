'use client';

import { useState, useEffect } from 'react';

// We dynamically pull from the bundled WASM API surface
let wasmInitPromise: Promise<any> | null = null;
let runLaadleCode: ((source: string) => string) | null = null;

export function Playground() {
  const [code, setCode] = useState<string>('// Welcome to the LaadleLang Playground!\nlaadle name hai "WASM"\nbol "Hello from " + name\n');
  const [output, setOutput] = useState<string>('Output will appear here...');
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    async function loadWasm() {
      try {
        // Dynamically import the JS WASM bridge locally
        // @ts-ignore: Next.js types can't statically analyze the Wasm runtime compilation
        const wasmModule = await import('../public/wasm/laadlelang.js');
        
        if (!wasmInitPromise) {
          // Tell the wrapper to fetch the statically served .wasm binary
          wasmInitPromise = wasmModule.default('/wasm/laadlelang_bg.wasm');
        }
        
        await wasmInitPromise;
        runLaadleCode = wasmModule.run_laadle_code;
        setIsLoading(false);
      } catch (e) {
        console.error('Failed to load WASM module', e);
        setOutput('Error: Failed to orchestrate WebAssembly VM.');
      }
    }
    loadWasm();
  }, []);

  const handleRun = () => {
    if (!runLaadleCode) return;
    
    setOutput('Running...');
    
    try {
      // Defer execution slightly to allow render paint
      setTimeout(() => {
        if (!runLaadleCode) return;
        const res = runLaadleCode(code);
        setOutput(res || 'Execution finished (no output).');
      }, 50);
    } catch (e) {
      setOutput(`Fatal Crash: ${e}`);
    }
  };

  return (
    <div className="flex flex-col group rounded-xl border bg-card text-card-foreground shadow-lg overflow-hidden my-12 w-full max-w-5xl mx-auto">
      {/* Editor Header */}
      <div className="flex items-center justify-between px-5 py-3 border-b bg-muted/80 backdrop-blur-sm">
        <div className="flex items-center gap-2">
          <div className="flex gap-2 mr-2">
            <div className="w-3 h-3 rounded-full bg-red-400" />
            <div className="w-3 h-3 rounded-full bg-amber-400" />
            <div className="w-3 h-3 rounded-full bg-emerald-400" />
          </div>
          <span className="text-sm font-semibold tracking-wide text-muted-foreground select-none">LaadleLang VM</span>
        </div>
        <button
          onClick={handleRun}
          disabled={isLoading}
          className="px-5 py-1.5 bg-blue-600 hover:bg-blue-700 active:bg-blue-800 disabled:bg-muted disabled:text-muted-foreground text-white text-sm font-bold rounded-lg transition-colors shadow-sm"
        >
          {isLoading ? 'Booting VM...' : '▶ Run'}
        </button>
      </div>

      <div className="grid md:grid-cols-2 divide-y md:divide-y-0 md:divide-x min-h-[450px]">
        {/* Editor Pane */}
        <textarea
          value={code}
          onChange={(e) => setCode(e.target.value)}
          spellCheck={false}
          className="w-full h-full p-6 font-mono text-[14px] leading-relaxed bg-transparent outline-none resize-none focus:ring-0"
          placeholder="Write your LaadleLang program here..."
        />
        
        {/* Output Pane */}
        <div className="p-6 bg-zinc-950 text-zinc-50 font-mono text-[14px] leading-relaxed overflow-y-auto whitespace-pre-wrap">
          <div className="text-zinc-500 mb-4 select-none">$ laadle run playground.la</div>
          <div className={`${output.includes('Error') || output.includes('Crash') ? 'text-red-400' : 'text-emerald-400'}`}>
            {output}
          </div>
        </div>
      </div>
    </div>
  );
}
