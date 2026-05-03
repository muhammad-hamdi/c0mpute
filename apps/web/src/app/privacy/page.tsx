export const metadata = { title: "privacy — c0mpute" };

export default function PrivacyPage() {
  return (
    <div className="max-w-3xl mx-auto px-6 py-16 space-y-8 text-sm leading-7">
      <header className="space-y-2">
        <h1 className="text-2xl font-bold accent">privacy</h1>
        <p className="comment">// last updated 2026-05-03 — v1 placeholder</p>
      </header>

      <Block title="what we collect">
        <p>
          The <code>c0mpute</code> CLI generates a local identity key and
          registers a CoinPay DID. Public DIDs, signed job receipts, and
          worker reputation are visible to the network — that&apos;s the
          point.
        </p>
        <p>
          Private keys, raw input data, and per-job logs stay on your
          machine unless you explicitly opt in (e.g. <code>--report</code>{" "}
          flags on doctor / crash dumps).
        </p>
      </Block>

      <Block title="what we don't collect">
        <ul className="list-disc pl-6 space-y-1">
          <li>No tracking pixels on this website. No third-party analytics scripts.</li>
          <li>No CLI telemetry by default.</li>
          <li>No long-lived bearer tokens — auth is signed-request envelopes per DIP-0007.</li>
        </ul>
      </Block>

      <Block title="your data on the network">
        <p>
          When you submit a job, its manifest (workload type, input hash,
          runtime image, max price) and the resulting receipt are visible to
          workers and validators. The actual input/output bytes are
          content-addressed and only fetched by parties involved in the
          job. For private workloads, encrypt inputs client-side; the
          c0mpute private trust tier (per the v1 PRD) supports this flow.
        </p>
      </Block>

      <Block title="cookies">
        <p>
          c0mpute.com uses session cookies only when you sign in to the
          dashboard (once that exists). No advertising cookies.
        </p>
      </Block>

      <Block title="contact">
        <p>
          Privacy concerns:{" "}
          <a href="mailto:privacy@c0mpute.com">privacy@c0mpute.com</a>
        </p>
      </Block>

      <p className="text-xs text-[var(--color-dim)] rule pt-6">
        Placeholder. A real privacy policy reviewed by counsel ships before
        public mainnet launch.
      </p>
    </div>
  );
}

function Block({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="space-y-2">
      <h2 className="text-base accent">[ {title} ]</h2>
      <div className="pl-5 space-y-2">{children}</div>
    </section>
  );
}
