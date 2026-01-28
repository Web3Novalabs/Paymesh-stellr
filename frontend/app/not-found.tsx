import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function NotFound() {
  return (
    <div 
      className="fixed inset-0 w-full h-full flex items-center justify-center overflow-hidden"
      style={{
        backgroundImage: 'url(/Stars.svg)',
        backgroundSize: 'cover',
        backgroundPosition: 'center',
        backgroundRepeat: 'no-repeat',
      }}
    >
      {/* Large 404 in background */}
      <div className="absolute inset-0 flex items-center justify-center overflow-hidden">
        <h1 
          className="font-anton font-black text-[250px] sm:text-[300px] md:text-[400px] lg:text-[500px] xl:text-[600px] select-none"
          style={{
            color: '#DFDFE0',
            opacity: 0.2,
            lineHeight: 0.7,
            fontWeight: 900,
            letterSpacing: '-0.02em',
          }}
        >
          404
        </h1>
      </div>

      <div className="relative z-10 px-6 max-w-[1151px] mx-auto flex flex-col items-center justify-center gap-8">
        <h3 
          className="font-anton font-bold text-xl sm:text-4xl md:text-5xl lg:text-[50px] xl:text-[60px] leading-tight uppercase text-center"
          style={{
            color: '#DFDFE0',
            lineHeight: 1.2,
            fontWeight: 700,
            letterSpacing: '0.03em',
          }}
        >
          We looked everywhere on-chain but couldn't find that page.
        </h3>
        
        <Link href="/">
          <Button variant="primary" size="lg">
            Return to Home
          </Button>
        </Link>
      </div>
    </div>
  );
}
