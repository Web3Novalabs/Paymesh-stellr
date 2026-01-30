"use client";
import { useState, useMemo } from "react";
import Image from "next/image";
interface Props {
  onEmailSubmit?: (email: string) => Promise<void>;
}
const validateEmail = (e: string) => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(e);
const genStars = () =>
  Array.from({ length: 104 }, (_, i) => ({
    id: i,
    x: Math.random() * 100,
    y: Math.random() * 100,
    r: Math.random() * 360,
  }));
export default function WaitlistHeroSection({ onEmailSubmit }: Props) {
  const [email, setEmail] = useState(""),
    [loading, setLoading] = useState(false),
    [status, setStatus] = useState<"idle" | "success" | "error">("idle"),
    [err, setErr] = useState("");
  const stars = useMemo(genStars, []);
  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.trim()) {
      setStatus("error");
      setErr("Email address is required");
      return;
    }
    if (!validateEmail(email)) {
      setStatus("error");
      setErr("Please enter a valid email address");
      return;
    }
    setLoading(true);
    setStatus("idle");
    try {
      if (onEmailSubmit) await onEmailSubmit(email);
      setEmail("");
      setStatus("success");
    } catch {
      setStatus("error");
      setErr("Unable to submit. Please try again.");
    } finally {
      setLoading(false);
    }
  };
  const navCls =
      "relative z-10 flex items-center px-4 sm:px-8 lg:px-[100px] py-5 h-[63px]",
    btnCls = "backdrop-blur-[5px] rounded hover:bg-white/15 transition-colors";
  return (
    <section className="relative w-full max-w-[1440px] h-[815px] mx-auto overflow-hidden bg-gradient-to-b from-[#030407] to-[#111827] flex flex-col">
      <div className="absolute inset-0 z-0">
        {stars.map((s) => (
          <div
            key={s.id}
            className="absolute w-1 h-1 bg-[#4B4B4B]"
            style={{
              left: `${s.x}%`,
              top: `${s.y}%`,
              transform: `rotate(${s.r}deg)`,
            }}
          />
        ))}
      </div>
      <nav
        className={`${navCls} justify-center bg-[rgba(5,7,11,0.1)] ${btnCls}`}
      >
        <div className="flex items-center gap-1 mx-auto">
          <Image
            src="/waitlist-icon.svg"
            alt="Paymesh logo"
            width={35}
            height={21}
          />
          <span className="text-white font-bold text-lg">Paymesh</span>
        </div>
      </nav>
      <main className="relative z-10 flex-1 flex flex-col justify-between items-center px-4 sm:px-8 lg:px-[100px] py-10 lg:py-[60px] gap-10 lg:gap-20">
        <div className="flex flex-col items-center gap-5 text-center">
          <h1 className="text-2xl sm:text-3xl lg:text-[34px] font-medium leading-tight bg-gradient-to-r from-[#DFDFE0] to-[#282B31] bg-clip-text text-transparent">
            Be the first to hear from us!
          </h1>
          <div className="flex gap-3">
            {[
              ["https://twitter.com", "X", "/Twitter-border.svg"],
              ["https://github.com", "Github", "/Github-border.svg"],
            ].map(([url, name, src]) => (
              <a
                key={name}
                href={url}
                target="_blank"
                rel="noopener noreferrer"
                className="p-2 rounded hover:bg-white/5 transition-colors"
                aria-label={`Follow us on ${name}`}
              >
                <Image src={src} alt={name} width={40} height={40} />
              </a>
            ))}
          </div>
        </div>
        <div className="flex items-center gap-3">
          <Image src="/waitlist-icon.svg" alt="" width={95} height={57} />
          <span className="text-white font-bold text-4xl sm:text-5xl lg:text-[48.6px]">
            Paymesh
          </span>
        </div>
        <form onSubmit={submit} className="w-full max-w-[500px]">
          <div
            className={`flex flex-col sm:flex-row gap-3 items-stretch sm:items-center bg-white/5 ${btnCls} p-3 sm:p-0 sm:pr-3`}
          >
            <input
              type="email"
              value={email}
              onChange={(e) => {
                setEmail(e.target.value);
                setStatus("idle");
                setErr("");
              }}
              placeholder="Enter email address"
              className="flex-1 bg-transparent px-4 sm:px-6 py-3 sm:py-4 text-white placeholder:text-[#8398AD] outline-none text-base"
              aria-label="Email address"
              aria-invalid={status === "error" ? "true" : "false"}
              disabled={loading}
            />
            <button
              type="submit"
              disabled={loading}
              className={`px-6 py-2.5 bg-white/10 ${btnCls} text-[#E2E2E2] text-sm disabled:opacity-50 disabled:cursor-not-allowed`}
              aria-label="Join waitlist"
            >
              {loading ? "Joining..." : "Join"}
            </button>
          </div>
          {status === "error" && (
            <p
              className="mt-2 text-red-400 text-sm"
              role="alert"
              aria-live="polite"
            >
              {err}
            </p>
          )}
          {status === "success" && (
            <p
              className="mt-2 text-green-400 text-sm"
              role="alert"
              aria-live="polite"
            >
              Thanks for joining! Check your email.
            </p>
          )}
        </form>
      </main>
      <nav className={`${navCls} justify-center gap-6 bg-[#0A1223]`}>
        {[
          ["/", "Home", "text-[#E2E2E2] underline"],
          ["#", "Waitlist", "text-[#8398AD]"],
        ].map(([href, text, cls]) => (
          <a
            key={text}
            href={href}
            className={`px-6 text-base hover:text-white transition-colors ${cls}`}
          >
            {text}
          </a>
        ))}
        <button
          type="button"
          className={`px-6 py-3 bg-white/10 ${btnCls} text-white text-sm`}
        >
          Launch App
        </button>
      </nav>
    </section>
  );
}
