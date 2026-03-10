import { memo } from "react";

interface StatusBarProps {
  connected: boolean;
  agentCount: number;
  sessionCount: number;
  activeView?: string;
}

const NAV_ITEMS = [
  { href: "/office/#office", label: "Office", id: "office" },
  { href: "/office/#fleet", label: "Fleet", id: "fleet" },
  { href: "/office/#mission", label: "Mission", id: "mission" },
  { href: "/", label: "Terminal", id: "terminal" },
  { href: "/dashboard", label: "Orbital", id: "orbital" },
];

export const StatusBar = memo(function StatusBar({ connected, agentCount, sessionCount, activeView = "office" }: StatusBarProps) {
  return (
    <header className="sticky top-0 z-20 flex flex-wrap items-center gap-x-4 gap-y-2 mx-4 sm:mx-6 mt-3 px-4 sm:px-6 py-2.5 rounded-2xl bg-black/50 backdrop-blur-xl border border-white/[0.06] shadow-[0_4px_30px_rgba(0,0,0,0.4)]">
      <h1 className="text-base sm:text-lg font-bold tracking-[4px] sm:tracking-[6px] text-cyan-400 uppercase whitespace-nowrap">
        {activeView === "fleet" ? "Fleet" : activeView === "mission" ? "Mission" : "Office"}
      </h1>

      <span className="flex items-center gap-1.5 text-sm text-white/70">
        <span className={`w-2 h-2 rounded-full flex-shrink-0 ${connected ? "bg-emerald-400 shadow-[0_0_6px_#4caf50]" : "bg-red-400 animate-pulse"}`} />
        {connected ? "LIVE" : "..."}
      </span>

      <span className="text-sm text-white/70 whitespace-nowrap">
        <strong className="text-cyan-400">{agentCount}</strong> agents
      </span>
      <span className="text-sm text-white/70 whitespace-nowrap">
        <strong className="text-purple-400">{sessionCount}</strong> rooms
      </span>

      <nav className="ml-auto flex items-center gap-3 sm:gap-4 text-sm">
        {NAV_ITEMS.map((item) => (
          <a
            key={item.id}
            href={item.href}
            className={`transition-colors whitespace-nowrap ${
              activeView === item.id
                ? "text-cyan-400 font-bold"
                : "text-white/50 hover:text-white/80"
            }`}
          >
            {item.label}
          </a>
        ))}
      </nav>
    </header>
  );
});
