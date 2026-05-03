import Link from "next/link";

export const metadata = { title: "docs — c0mpute" };

export default function DocsPage() {
  return (
    <div className="max-w-3xl mx-auto px-6 py-16 space-y-10">
      <header className="space-y-2">
        <h1 className="text-2xl font-bold accent">docs</h1>
        <p className="comment">// references for the cli, modules, and protocol</p>
      </header>

      <section className="space-y-3">
        <h2 className="text-lg accent">[ cli ]</h2>
        <ul className="space-y-2 pl-5 text-sm leading-6">
          <li>
            <code>c0mpute</code> — umbrella binary; runs <code>c0mpute &lt;module&gt; &lt;args&gt;</code>{" "}
            and the worker / job lifecycle.
          </li>
          <li>
            <code>coinpay</code> — DID, wallet, escrow, receipts, reputation.
          </li>
          <li>
            <code>infernet</code> — AI inference workload runner.
          </li>
          <li className="text-[var(--color-dim)]">
            See{" "}
            <a href="https://github.com/profullstack/c0mpute/blob/main/docs/c0mpute-v1.md">
              docs/c0mpute-v1.md
            </a>{" "}
            in the repo for the full v1 PRD.
          </li>
        </ul>
      </section>

      <section className="space-y-3">
        <h2 className="text-lg accent">[ modules ]</h2>
        <ul className="space-y-2 pl-5 text-sm leading-6">
          <li>
            <span className="accent">transcode</span> — FFmpeg, in-process.
            Workloads: <code>ffmpeg.transcode</code>.
          </li>
          <li>
            <span className="accent">coinpay</span> — DID + payments. Service.
          </li>
          <li>
            <span className="accent">infernet</span> — AI inference. Workloads:{" "}
            <code>infernet.inference</code>.
          </li>
          <li className="text-[var(--color-dim)]">
            Each module's <code>module.toml</code> manifest lives in{" "}
            <code>plugins/&lt;id&gt;/</code> in the repo.
          </li>
        </ul>
      </section>

      <section className="space-y-3">
        <h2 className="text-lg accent">[ design proposals ]</h2>
        <p className="text-sm text-[var(--color-dim)]">
          Improvement plans are tracked in <code>dips/</code>. Highlights:
        </p>
        <ul className="space-y-1 pl-5 text-sm leading-6">
          <li>
            <a href="https://github.com/profullstack/c0mpute/blob/main/dips/0005-c0mpute-rebrand.md">
              DIP-0005
            </a>{" "}
            — c0mpute.com rebrand, three-CLI architecture
          </li>
          <li>
            <a href="https://github.com/profullstack/c0mpute/blob/main/dips/0006-module-model.md">
              DIP-0006
            </a>{" "}
            — module model, manifest, distribution
          </li>
          <li>
            <a href="https://github.com/profullstack/c0mpute/blob/main/dips/0007-coinpay-did-identity.md">
              DIP-0007
            </a>{" "}
            — CoinPay DID as the identity layer
          </li>
        </ul>
      </section>

      <section className="rule pt-8 text-sm text-[var(--color-dim)]">
        <p>
          → <Link href="/getting-started">getting-started</Link>{" "}
          → <Link href="/contact">contact</Link>
        </p>
      </section>
    </div>
  );
}
