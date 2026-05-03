export const metadata = { title: "getting-started — c0mpute" };

export default function GettingStartedPage() {
  return (
    <div className="max-w-3xl mx-auto px-6 py-16 space-y-10">
      <header className="space-y-2">
        <h1 className="text-2xl font-bold accent">getting-started</h1>
        <p className="comment">// install → identity → register → run</p>
      </header>

      <Section number="1" label="install the cli stack">
        <pre className="bg-[var(--color-card)] border border-[var(--color-rule)] rounded p-4 overflow-x-auto text-sm leading-6">
          <span className="prompt">curl -fsSL https://c0mpute.com/install.sh | sh</span>
        </pre>
        <p className="text-sm text-[var(--color-dim)]">
          Installs three binaries into <code>~/.c0mpute/bin</code>:{" "}
          <span className="accent">c0mpute</span>,{" "}
          <span className="accent">coinpay</span>,{" "}
          <span className="accent">infernet</span>. No sudo. Self-upgrades on
          its own schedule.
        </p>
      </Section>

      <Section number="2" label="create a coinpay did">
        <pre className="bg-[var(--color-card)] border border-[var(--color-rule)] rounded p-4 overflow-x-auto text-sm leading-6">
          <span className="prompt">c0mpute coinpay did create</span>
        </pre>
        <p className="text-sm text-[var(--color-dim)]">
          Generates a key + registers <code>did:coinpay:user:&lt;id&gt;</code>.
          The DID anchors your reputation, payments, and signed receipts.
        </p>
      </Section>

      <Section number="3" label="run a worker">
        <pre className="bg-[var(--color-card)] border border-[var(--color-rule)] rounded p-4 overflow-x-auto text-sm leading-6">
          {[
            "c0mpute coinpay did create --role worker",
            "c0mpute worker register",
            "c0mpute worker start --gpu",
          ]
            .map((c, i) => (
              <span key={i}>
                <span className="prompt">{c}</span>
                {"\n"}
              </span>
            ))}
        </pre>
      </Section>

      <Section number="4" label="submit a job">
        <pre className="bg-[var(--color-card)] border border-[var(--color-rule)] rounded p-4 overflow-x-auto text-sm leading-6">
          {[
            "c0mpute transcode submit input.mov --preset hls --max-price 1.25",
            "c0mpute infernet run prompts.jsonl --model qwen",
            "c0mpute job status <job-id>",
          ].map((c, i) => (
            <span key={i}>
              <span className="prompt">{c}</span>
              {"\n"}
            </span>
          ))}
        </pre>
      </Section>

      <Section number="5" label="interactive tui">
        <pre className="bg-[var(--color-card)] border border-[var(--color-rule)] rounded p-4 overflow-x-auto text-sm leading-6">
          <span className="prompt">c0mpute tui</span>
        </pre>
        <p className="text-sm text-[var(--color-dim)]">
          Live worker / job dashboard, terminal-native (react-blessed).
        </p>
      </Section>

      <Section number="6" label="check the stack">
        <pre className="bg-[var(--color-card)] border border-[var(--color-rule)] rounded p-4 overflow-x-auto text-sm leading-6">
          <span className="prompt">c0mpute doctor</span>
        </pre>
      </Section>
    </div>
  );
}

function Section({
  number,
  label,
  children,
}: {
  number: string;
  label: string;
  children: React.ReactNode;
}) {
  return (
    <section className="space-y-3">
      <h2 className="text-lg">
        <span className="accent">[{number}]</span>{" "}
        <span className="text-[var(--color-fg)]">{label}</span>
      </h2>
      <div className="space-y-2 pl-5">{children}</div>
    </section>
  );
}
