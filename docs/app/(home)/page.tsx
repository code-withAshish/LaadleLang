import Link from 'next/link';

export default function HomePage() {
  return (
    <main className="flex flex-col flex-1 items-center justify-center text-center px-4 py-24 min-h-[calc(100vh-4rem)]">
      <div className="absolute inset-0 bg-gradient-to-b from-blue-900/10 to-transparent -z-10 pointer-events-none" />
      <div className="max-w-4xl mx-auto flex flex-col items-center mt-20">
        <div className="inline-flex items-center rounded-full border px-3 py-1 text-sm font-medium mb-8 bg-background shadow-sm hover:shadow-md transition-all cursor-pointer">
          <span className="flex rounded-full bg-blue-500 w-2 h-2 mr-2 animate-pulse" />
          LaadleLang v0.1 is Live
        </div>
        
        <h1 className="text-5xl md:text-7xl font-extrabold tracking-tight mb-6 bg-gradient-to-r from-blue-600 via-purple-600 to-indigo-600 bg-clip-text text-transparent pb-2">
          Programming, <br className="hidden md:block" /> Reimagined.
        </h1>
        
        <p className="text-xl md:text-2xl text-muted-foreground mb-10 max-w-2xl leading-relaxed">
          The blazingly fast, deeply intuitive, and beautifully structured language running on a custom deterministic Virtual Machine.
        </p>
        
        <div className="flex flex-col sm:flex-row gap-4 items-center">
          <Link 
            href="/playground" 
            className="flex items-center justify-center rounded-lg bg-blue-600 px-8 py-3.5 text-sm font-medium text-white shadow-lg transition-transform hover:-translate-y-1 hover:shadow-blue-500/25 active:scale-95"
          >
            Launch Playground IDE
          </Link>
          <Link 
            href="/docs" 
            className="flex items-center justify-center rounded-lg border bg-background px-8 py-3.5 text-sm font-medium shadow-sm transition-colors hover:bg-muted"
          >
            Read Documentation
          </Link>
        </div>
      </div>
      
      <div className="mt-32 grid md:grid-cols-3 gap-8 text-left max-w-5xl mx-auto w-full px-6 mb-20">
        <FeatureCard 
          title="Custom Virtual Machine" 
          description="Powered by an isolated Bytecode Compiler tracking explicit memory layouts and stateful CallFrames." 
          icon="⚡" 
        />
        <FeatureCard 
          title="Failover Execution" 
          description="Robust error unwinding using natively optimized koshish and pakad block handlers to trap crashes." 
          icon="🛡️" 
        />
        <FeatureCard 
          title="Beautiful Tooling" 
          description="Flawlessly generated ASTs, 100% test coverage, and a fully interactive documentation framework." 
          icon="✨" 
        />
      </div>
    </main>
  );
}

function FeatureCard({ title, description, icon }: { title: string, description: string, icon: string }) {
  return (
    <div className="flex flex-col p-6 rounded-2xl border bg-card text-card-foreground shadow-sm hover:shadow-md hover:border-blue-500/30 transition-all duration-300">
      <div className="text-3xl mb-4 p-3 bg-muted w-fit rounded-lg">{icon}</div>
      <h3 className="font-semibold text-xl mb-2">{title}</h3>
      <p className="text-muted-foreground leading-relaxed">{description}</p>
    </div>
  )
}
