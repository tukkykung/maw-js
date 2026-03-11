import { useState, useRef, useCallback, useEffect } from "react";

type SpeechRec = {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  start(): void;
  stop(): void;
  abort(): void;
  onresult: ((e: any) => void) | null;
  onend: (() => void) | null;
  onerror: ((e: any) => void) | null;
};

function createRecognition(): SpeechRec | null {
  const W = window as any;
  const Ctor = W.SpeechRecognition || W.webkitSpeechRecognition;
  if (!Ctor) return null;
  return new Ctor() as SpeechRec;
}

export const hasWebSpeech = !!createRecognition();

interface SpeechState {
  listening: boolean;
  transcript: string;
  target: string | null; // which agent target is being spoken to
}

// Singleton — only one mic active at a time
let activeSpeech: {
  recognition: SpeechRec;
  setState: (s: Partial<SpeechState>) => void;
  onDone: (text: string) => void;
} | null = null;

export function useSpeech(send: (msg: object) => void) {
  const [state, setState] = useState<SpeechState>({ listening: false, transcript: "", target: null });
  const stateRef = useRef(state);
  stateRef.current = state;
  const sendRef = useRef(send);
  sendRef.current = send;

  const startListening = useCallback((target: string) => {
    // Stop any existing
    if (activeSpeech) {
      activeSpeech.recognition.abort();
      activeSpeech = null;
    }

    const rec = createRecognition();
    if (!rec) return false;

    rec.continuous = false;
    rec.interimResults = true;
    rec.lang = "th-TH";

    let finalText = "";

    rec.onresult = (e: any) => {
      let interim = "";
      let final = "";
      for (let i = 0; i < e.results.length; i++) {
        const r = e.results[i];
        if (r.isFinal) final += r[0].transcript;
        else interim += r[0].transcript;
      }
      if (final) finalText = final;
      setState(prev => ({ ...prev, transcript: interim || final }));
    };

    rec.onend = () => {
      setState({ listening: false, transcript: "", target: null });
      if (finalText.trim()) {
        sendRef.current({ type: "send", target, text: finalText.trim() });
        setTimeout(() => sendRef.current({ type: "send", target, text: "\r" }), 50);
      }
      activeSpeech = null;
    };

    rec.onerror = (e: any) => {
      console.warn("Speech error:", e.error);
      setState({ listening: false, transcript: "", target: null });
      activeSpeech = null;
    };

    activeSpeech = { recognition: rec, setState: () => {}, onDone: () => {} };
    setState({ listening: true, transcript: "", target });
    rec.start();
    return true;
  }, []);

  const stopListening = useCallback(() => {
    if (activeSpeech) {
      activeSpeech.recognition.stop();
    }
  }, []);

  return { ...state, startListening, stopListening };
}
