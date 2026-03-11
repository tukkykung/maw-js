import { memo } from "react";
import { roomStyle } from "../lib/constants";

interface SpeechOverlayProps {
  listening: boolean;
  transcript: string;
  target: string | null;
  agentName?: string;
  agentSession?: string;
  onStop: () => void;
}

export const SpeechOverlay = memo(function SpeechOverlay({
  listening, transcript, target, agentName, agentSession, onStop,
}: SpeechOverlayProps) {
  if (!listening || !target) return null;

  const rs = agentSession ? roomStyle(agentSession) : { accent: "#fbbf24" };
  const displayName = agentName?.replace(/-oracle$/, "").replace(/-/g, " ") || target;

  return (
    <div
      className="fixed inset-0 z-[60] flex flex-col items-center justify-center"
      style={{ background: "rgba(0,0,0,0.88)", backdropFilter: "blur(8px)" }}
      onClick={onStop}
    >
      {/* Pulsing rings */}
      <div className="relative w-40 h-40">
        <div className="absolute inset-0 rounded-full animate-ping" style={{ background: `${rs.accent}12`, animationDuration: "1.5s" }} />
        <div className="absolute inset-4 rounded-full animate-ping" style={{ background: `${rs.accent}18`, animationDuration: "1.2s" }} />
        <div
          className="absolute inset-8 rounded-full flex items-center justify-center cursor-pointer active:scale-90"
          style={{ background: rs.accent, boxShadow: `0 0 60px ${rs.accent}80` }}
        >
          <svg width={40} height={40} viewBox="0 0 24 24" fill="none" stroke="#000" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
            <rect x={9} y={1} width={6} height={11} rx={3} />
            <path d="M19 10v1a7 7 0 01-14 0v-1M12 18v4M8 22h8" />
          </svg>
        </div>
      </div>

      {/* Target agent */}
      <div className="mt-8 flex items-center gap-2.5">
        <div className="w-3 h-3 rounded-full" style={{ background: rs.accent, boxShadow: `0 0 8px ${rs.accent}` }} />
        <span className="text-[16px] font-semibold" style={{ color: rs.accent }}>{displayName}</span>
      </div>

      {/* Live transcript */}
      <div className="mt-5 px-10 text-center min-h-[3em] max-w-md">
        <p className="text-white/80 text-lg leading-relaxed">{transcript || "Listening..."}</p>
      </div>

      {/* Hint */}
      <p className="mt-8 text-white/15 text-[12px] font-mono">Tap anywhere to send</p>
    </div>
  );
});
