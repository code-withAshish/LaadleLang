import Link from 'next/link';

export default function HomePage() {
  return (
    <main className="flex flex-col flex-1 items-center justify-center text-center px-4 py-24 min-h-[calc(100vh-4rem)]">
      <div className="absolute inset-0 bg-gradient-to-b from-[#1a1a2e]/30 to-[#16213e]/10 -z-10 pointer-events-none" />
      <div className="max-w-4xl mx-auto flex flex-col items-center mt-12">
        <div className="inline-flex items-center rounded-full border border-gray-700 px-4 py-1.5 text-sm font-semibold mb-10 bg-[#1e1e1e] shadow-sm hover:shadow-md transition-all cursor-pointer">
          <span className="flex rounded-full bg-orange-500 w-2.5 h-2.5 mr-2 animate-pulse" />
          <span className="text-gray-300">Scene set hai bhai, v1.0 Live hai! 🔥</span>
        </div>
        
        <h1 className="text-5xl md:text-7xl font-black tracking-tight mb-6 bg-gradient-to-br from-orange-400 via-pink-500 to-purple-600 bg-clip-text text-transparent pb-3">
          Coding, <br className="hidden md:block" /> Laadle Style 😎
        </h1>
        
        <p className="text-xl md:text-2xl text-gray-400 mb-12 max-w-2xl leading-relaxed font-medium">
          A toy programming language written in Rust for the absolute legends. Because writing code should feel like talking to your homies.
        </p>

        {/* Code Snippet Display */}
        <div className="bg-[#1e1e1e] p-6 rounded-2xl text-left font-mono text-[15px] sm:text-base mb-14 shadow-2xl border border-gray-700 w-full max-w-2xl overflow-x-auto text-[#abb2bf] transform transition-transform hover:scale-[1.02]">
          <p><span className="text-[#c678dd]">laadle</span> <span className="text-[#e06c75]">name</span> <span className="text-[#c678dd]">hai</span> <span className="text-[#98c379]">"LaadleLang"</span></p>
          <p><span className="text-[#c678dd]">agar laadle</span> name <span className="text-[#56b6c2]">barabar hai</span> <span className="text-[#98c379]">"LaadleLang"</span> <span className="text-[#c678dd]">toh</span></p>
          <p className="pl-8"><span className="text-[#c678dd]">laadle bol</span> <span className="text-[#98c379]">"Bhai language toh aag hai! 🔥"</span></p>
          <p><span className="text-[#c678dd]">bas</span></p>
        </div>
        
        <div className="flex flex-col sm:flex-row gap-5 items-center">
          <Link 
            href="/playground/index.html" 
            className="flex items-center justify-center rounded-xl bg-gradient-to-r from-orange-500 to-pink-600 px-8 py-4 text-base font-bold text-white shadow-xl transition-transform hover:-translate-y-1 hover:shadow-orange-500/25 active:scale-95"
          >
            Playground Khol 🚀
          </Link>
          <Link 
            href="/docs" 
            className="flex items-center justify-center rounded-xl border-2 border-gray-700 bg-[#161b22] px-8 py-4 text-base font-bold text-white shadow-sm transition-colors hover:bg-gray-800"
          >
            Docs Padhle Bhai 📚
          </Link>
        </div>
      </div>
      
      <div className="mt-32 grid md:grid-cols-3 gap-8 text-left max-w-5xl mx-auto w-full px-6 mb-24">
        <FeatureCard 
          title="Built for the Boys 💪" 
          description="Under the hood, it's a ridiculously fast Custom Virtual Machine written entirely in Rust that doesn't mess around." 
          icon="⚡" 
        />
        <FeatureCard 
          title="Koshish-Pakad 🛡️" 
          description="Built-in error handling syntax using 'koshish' and 'pakad'. Because even chads make mistakes sometimes." 
          icon="🧠" 
        />
        <FeatureCard 
          title="Zero Khit-pit 🚀" 
          description="Comes with a blazing fast offline WebAssembly Playground, Native REPL, and a zero-dependency portable executable." 
          icon="✨" 
        />
      </div>

      <div className="mt-4 text-gray-500 text-sm font-bold tracking-widest uppercase">
        Built with ❤️ in Rust for the sheer fun of it.
      </div>
    </main>
  );
}

function FeatureCard({ title, description, icon }: { title: string, description: string, icon: string }) {
  return (
    <div className="flex flex-col p-8 rounded-3xl border border-gray-800 bg-[#1a1e26] text-card-foreground shadow-xl hover:shadow-2xl hover:border-pink-500/40 hover:-translate-y-2 transition-all duration-300">
      <div className="text-4xl mb-5 p-4 bg-[#282c34] w-fit rounded-xl border border-gray-800 shadow-inner">{icon}</div>
      <h3 className="font-bold text-2xl mb-3 text-white">{title}</h3>
      <p className="text-gray-400 leading-relaxed font-medium">{description}</p>
    </div>
  )
}
