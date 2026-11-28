import { Faq } from "@/components/Faq";
import { FinalCta } from "@/components/FinalCta";
import { Footer } from "@/components/Footer";
import { Header } from "@/components/Header";
import { Hero } from "@/components/Hero";
import { HowItWorks } from "@/components/HowItWorks";
import { PlatformStrip } from "@/components/PlatformStrip";
import { StarButton } from "@/components/StarButton";
import { Switches } from "@/components/Switches";
import { WhyGrid } from "@/components/WhyGrid";

export default function Home() {
  return (
    <>
      <Header star={<StarButton />} />
      <main className="flex-1">
        <Hero />
        <PlatformStrip />
        <HowItWorks />
        <Switches />
        <WhyGrid />
        <Faq />
        <FinalCta />
      </main>
      <Footer />
    </>
  );
}
