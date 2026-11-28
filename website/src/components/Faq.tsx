import FaqSection, { type FaqData } from "@/components/ui/habit-faq-scroller";
import { links } from "@/content/site";

// Every answer here comes from the README. Keep about four cards per row so the loop never shows a gap.
const faqData: FaqData = {
  mainTitle: "Questions",
  mainSubtitle: (
    <>
      Short answers to what people ask before installing. Something else?{" "}
      <a className="text-ink underline decoration-line-strong underline-offset-4 hover:decoration-seal" href={`${links.repo}/issues`}>
        Open an issue
      </a>
      .
    </>
  ),
  rows: [
    {
      id: "row1",
      speed: "60s",
      direction: "left",
      faqItems: [
        {
          id: "record",
          question: "Does Clackboard record what I type?",
          answer:
            "No. It only learns which key position went down or up, never the character. Nothing is saved to disk or sent anywhere.",
        },
        {
          id: "lag",
          question: "Will I hear a delay?",
          answer:
            "No. Sounds play from a small low-latency mixer inside the app, and each clip is trimmed so it starts the moment the key moves.",
        },
        {
          id: "which",
          question: "Which switches are included?",
          answer:
            "26: Cherry MX Blue, Brown, Black, Red and Speed Silver, IBM Model M, Holy Panda, NovelKeys Cream, Gateron Ink, Topre, a typewriter and more.",
        },
        {
          id: "own",
          question: "Can I add my own sounds?",
          answer: "Yes. Press Add your own in the panel and drop in any Mechvibes sound pack folder. It shows up in the list.",
        },
      ],
    },
    {
      id: "row2",
      speed: "45s",
      direction: "right",
      faqItems: [
        {
          id: "macos",
          question: "What does macOS ask for?",
          answer:
            "Input Monitoring, once, so it can hear which keys you press. Builds aren't notarized yet, so the first time you open it, click Open Anyway in Privacy & Security.",
        },
        {
          id: "windows",
          question: "Why does Windows warn me?",
          answer:
            "The installer isn't signed by Microsoft yet, so SmartScreen warns you the first time. Click More info, then Run anyway.",
        },
        {
          id: "linux",
          question: "What do I need on Linux?",
          answer:
            "Read access to your keyboards. Run sudo usermod -aG input $USER, then log out and back in. It works on X11 and Wayland.",
        },
        {
          id: "password",
          question: "Does it click in password fields?",
          answer: "On a Mac it can't: macOS hides keys from every app while you type a password, so Clackboard stays silent there.",
        },
      ],
    },
    {
      id: "row3",
      speed: "70s",
      direction: "left",
      faqItems: [
        {
          id: "free",
          question: "Is it free?",
          answer: "Yes. Clackboard is MIT licensed and all of the code is on GitHub. Some sound packs keep their own GPL licenses.",
        },
        {
          id: "clean",
          question: "What's the keyboard lock for?",
          answer:
            "Cleaning. Lock the keys for a set time and wipe them without typing into whatever is open. Your mouse keeps working, and Ctrl + Alt + U always unlocks.",
        },
        {
          id: "hushboard",
          question: "How is this related to Hushboard?",
          answer:
            "Clackboard is a fork of Hushboard, the open-source keyboard lock by Tekcify. The lock and panel come from it; Clackboard adds the sounds.",
        },
        {
          id: "updates",
          question: "How do I update?",
          answer: "Download the newest release and install it over the old one. In-app updates are coming.",
        },
        {
          id: "help",
          question: "How can I help?",
          answer: "Report bugs on GitHub, test it on your own hardware, or record a switch and share the pack.",
        },
      ],
    },
  ],
};

export function Faq() {
  return (
    <section id="faq" aria-label="Frequently asked questions" className="flex justify-center py-16 sm:py-24">
      <FaqSection data={faqData} />
    </section>
  );
}
