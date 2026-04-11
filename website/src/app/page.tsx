import { CopyCommand } from "./copy-command";
import { HeroArtwork } from "./hero-artwork";
import { ThemeToggle } from "./theme-toggle";
import Link from "next/link";

export default function Home() {
  return (
    <main className="min-h-screen overflow-x-hidden">
      <Nav />
      <Hero />

      <Features />
      <ConfigSection />
      <Integrations />
      <AntiPatterns />
      <Architecture />
      <CTA />
      <Footer />
    </main>
  );
}

/* ─── Nav ─── */

function Nav() {
  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-background/80 backdrop-blur-sm border-b border-border">
      <div className="px-6 h-12 flex items-center">
        <div className="shrink-0 lg:w-[39%]">
          <Link href="/">
            <BrandLogo />
          </Link>
        </div>
        <div className="hidden md:flex items-center h-full">
          {[
            { label: "Features", href: "#features" },
            { label: "Integrations", href: "#integrations" },
            { label: "Protocol", href: "https://github.com/sambam60/blackwall/blob/main/spec/PROTOCOL.md", external: true },
            { label: "GitHub", href: "https://github.com/sambam60/blackwall", external: true },
          ].map((link) => (
            <a
              key={link.label}
              href={link.href}
              {...(link.external ? { target: "_blank", rel: "noopener noreferrer" } : {})}
              className="h-full flex items-center px-5 text-[11px] font-medium uppercase tracking-[0.08em] text-muted hover:text-foreground border-r border-border first:border-l transition-colors"
            >
              {link.label}
            </a>
          ))}
        </div>
        <div className="ml-auto flex items-center gap-4">
          <a href="#features" className="md:hidden text-[11px] font-medium uppercase tracking-[0.08em] text-muted hover:text-foreground transition-colors">Features</a>
          <a href="https://github.com/sambam60/blackwall" target="_blank" rel="noopener noreferrer" className="md:hidden text-[11px] font-medium uppercase tracking-[0.08em] text-muted hover:text-foreground transition-colors">GitHub</a>
          <ThemeToggle />
        </div>
      </div>
    </nav>
  );
}

/* ─── Hero ─── */

function Hero() {
  return (
    <section className="relative min-h-[100svh] overflow-hidden">
      <HeroArtwork />

      <div className="relative z-10 min-h-[100svh] flex items-center justify-center lg:justify-end px-6">
        <div className="w-full lg:w-[62%] lg:pl-8 pt-24 pb-16 lg:py-32">
          <div className="inline-flex items-center gap-2 text-xs text-muted/80 rounded-full py-1.5 mb-8 font-mono">
            <span className="w-1.5 h-1.5 rounded-full bg-green animate-pulse" />
            OPEN SOURCE &middot; v0.1.0
          </div>
          <h1 className="text-[clamp(2.5rem,5.5vw,4.5rem)] font-bold tracking-[-0.03em] leading-[1.08] mb-6">
            The execution firewall
            <br />
            <span className="text-muted/70">for agents</span>
          </h1>
          <p className="text-lg text-muted/80 max-w-[500px] mb-10 leading-relaxed">
            File writes, shell commands, network requests, MCP tool calls. Every action passes through a deterministic policy engine before it executes. Stop your agents from going rogue.
          </p>
          <CopyCommand
            commands={[
              { id: "cargo", label: "cargo", command: "cargo install --path crates/blackwall-cli" },
              { id: "brew", label: "brew", command: "brew tap sambam60/blackwall && brew install blackwall" },
              {
                id: "curl",
                label: "curl",
                command:
                  "curl -fsSL -o blackwall-install.sh https://raw.githubusercontent.com/sambam60/blackwall/main/install.sh && bash blackwall-install.sh",
              },
            ]}
          />
        </div>
      </div>
    </section>
  );
}

/* ─── Features ─── */

function Features() {
  return (
    <section id="features" className="py-24 px-6">
      <div className="max-w-[1200px] mx-auto">
        <p className="text-xs font-mono text-muted-darker uppercase tracking-widest mb-4">Features</p>

        <div className="grid md:grid-cols-2 gap-3 mt-10">
          {/* 01 - Shell Interception (large) */}
          <FeatureCard num="01" title="Shell Interception" subtitle="Every command, evaluated." className="md:row-span-2">
            <p className="text-muted text-sm leading-relaxed mb-6">
              PATH-prepended shims catch every agent command before execution. Git, curl, npm, python, and more. The real binary only runs after policy approval.
            </p>
            <div className="bg-background rounded-lg border border-border p-4 font-mono text-xs space-y-1.5 overflow-x-auto">
              <LogLine time="14:23:01" icon="✓" color="text-green" label="shell.exec" value="git status" />
              <LogLine time="14:23:03" icon="✓" color="text-green" label="shell.exec" value="cargo build" />
              <LogLine time="14:23:09" icon="✓" color="text-green" label="shell.exec" value="npm install express" />
              <LogLine time="14:23:12" icon="⏸" color="text-amber" label="shell.exec" value="rm -rf node_modules" />
              <LogLine time="14:23:15" icon="✗" color="text-red" label="shell.exec" value="sudo rm -rf /" />
            </div>
          </FeatureCard>

          {/* 02 - MCP Proxy */}
          <FeatureCard num="02" title="MCP Tool Proxy" subtitle="Intercept every tool call.">
            <p className="text-muted text-sm leading-relaxed mb-6">
              Stdio man-in-the-middle on JSON-RPC <code className="text-foreground/70 bg-surface-tint px-1.5 py-0.5 rounded text-xs">tools/call</code>. Every MCP tool invocation evaluated before reaching the server.
            </p>
            <div className="bg-background rounded-lg border border-border p-4 font-mono text-xs space-y-1.5 overflow-x-auto">
              <LogLine time="14:23:20" icon="·" color="text-muted" label="mcp.tool_call" value="read_file" />
              <LogLine time="14:23:22" icon="✗" color="text-red" label="mcp.tool_call" value="execute_command" />
              <LogLine time="14:23:25" icon="·" color="text-muted" label="mcp.tool_call" value="search_files" />
            </div>
          </FeatureCard>

          {/* 03 - Policy Engine */}
          <FeatureCard num="03" title="Policy Engine" subtitle="Deterministic decisions.">
            <p className="text-muted text-sm leading-relaxed mb-5">
              Three built-in profiles: default, strict, and permissive. Or write your own in YAML. Sub-millisecond evaluation per action.
            </p>
            <div className="flex flex-wrap gap-2">
              {["default", "strict", "permissive", "custom.yaml"].map((p) => (
                <span key={p} className="text-[11px] font-mono px-2.5 py-1 rounded-md bg-surface-tint border border-border-light text-muted">{p}</span>
              ))}
            </div>
          </FeatureCard>

          {/* 04 - Human Escalation (large) */}
          <FeatureCard num="04" title="Human Escalation" subtitle="Humans stay in the loop." className="md:row-span-2">
            <p className="text-muted text-sm leading-relaxed mb-6">
              When the gateway is uncertain, it pauses and prompts the human inline. Allow, deny, or end the session entirely.
            </p>
            <div className="bg-background rounded-lg border border-border p-4 font-mono text-xs overflow-x-auto">
              <div className="text-amber mb-1.5 min-w-max">⏸ PAUSE: confirmation required</div>
              <div className="border-l-2 border-border-light pl-3 space-y-1 text-muted mb-3">
                <div>shell.exec <span className="text-foreground/70">rm -rf node_modules</span></div>
                <div>reason: <span className="text-foreground/50">matches confirmation rule &apos;rm -rf&apos;</span></div>
              </div>
              <div className="flex flex-wrap gap-2 mt-3">
                <span className="px-3 py-1.5 rounded bg-green/10 text-green border border-green/20 text-[11px]">allow (a)</span>
                <span className="px-3 py-1.5 rounded bg-red/10 text-red border border-red/20 text-[11px]">deny (d)</span>
                <span className="px-3 py-1.5 rounded bg-surface-tint text-muted border border-border-light text-[11px]">end session (x)</span>
              </div>
            </div>
          </FeatureCard>

          {/* 05 - Temporal Anti-Patterns */}
          <FeatureCard num="05" title="Anti-Pattern Detection" subtitle="Behavioral threat modeling.">
            <p className="text-muted text-sm leading-relaxed mb-5">
              Sliding window detection catches credential harvesting, sandbox probing, self-modification, data exfiltration, and history tampering.
            </p>
            <div className="grid grid-cols-1 min-[420px]:grid-cols-2 gap-2">
              {[
                { label: "credential_harvesting", action: "HALT" },
                { label: "sandbox_probing", action: "HALT" },
                { label: "self_modification", action: "HALT" },
                { label: "public_exfiltration", action: "PAUSE" },
              ].map((p) => (
                <div key={p.label} className="flex items-center justify-between bg-background rounded-md border border-border px-3 py-2 min-w-0">
                  <span className="text-[10px] font-mono text-muted truncate mr-2">{p.label}</span>
                  <span className={`text-[10px] font-mono shrink-0 ${p.action === "HALT" ? "text-red" : "text-amber"}`}>{p.action}</span>
                </div>
              ))}
            </div>
          </FeatureCard>

          {/* 06 - Circuit Breaker */}
          <FeatureCard num="06" title="Circuit Breaker" subtitle="Session-level kill switch.">
            <p className="text-muted text-sm leading-relaxed mb-5">
              Critical pattern matches trip a circuit breaker, halting all further actions. No recovery without human restart. Rate limiting built in.
            </p>
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2 text-xs font-mono">
                <span className="w-2 h-2 rounded-full bg-green" />
                <span className="text-muted">Active</span>
              </div>
              <svg width="24" height="8" className="text-muted-darker"><path d="M0 4h24" stroke="currentColor" strokeWidth="1" strokeDasharray="2 2"/></svg>
              <div className="flex items-center gap-2 text-xs font-mono">
                <span className="w-2 h-2 rounded-full bg-amber" />
                <span className="text-muted">Paused</span>
              </div>
              <svg width="24" height="8" className="text-muted-darker"><path d="M0 4h24" stroke="currentColor" strokeWidth="1" strokeDasharray="2 2"/></svg>
              <div className="flex items-center gap-2 text-xs font-mono">
                <span className="w-2 h-2 rounded-full bg-red" />
                <span className="text-muted">Halted</span>
              </div>
            </div>
          </FeatureCard>

          {/* 07 - Audit Trail */}
          <FeatureCard num="07" title="Full Audit Trail" subtitle="Every decision, recorded." className="md:col-span-2">
            <p className="text-muted text-sm leading-relaxed mb-6">
              Append-only JSONL logs for every action evaluation. Session ID, timestamps, decisions, reasons, sub-millisecond latency. Tail live or query after.
            </p>
            <div className="bg-background rounded-lg border border-border p-4 font-mono text-[11px] space-y-1 overflow-x-auto whitespace-nowrap">
              <div className="text-muted-darker">~/.blackwall/logs/7f3a28.jsonl</div>
              <div className="mt-2 text-foreground/60">
                {`{"ts":"2025-01-15T14:23:01Z","session":"7f3a28","seq":1,"tool":"shell","op":"exec","target":"git status","decision":"`}<span className="text-green">allow</span>{`","latency_us":42}`}
              </div>
              <div className="text-foreground/60">
                {`{"ts":"2025-01-15T14:23:15Z","session":"7f3a28","seq":2,"tool":"shell","op":"exec","target":"sudo rm -rf /","decision":"`}<span className="text-red">deny</span>{`","reason":"command 'sudo' matches deny rule","latency_us":18}`}
              </div>
            </div>
          </FeatureCard>
        </div>
      </div>
    </section>
  );
}

function FeatureCard({
  num,
  title,
  subtitle,
  children,
  className = "",
}: {
  num: string;
  title: string;
  subtitle: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div className={`bg-card border border-border rounded-xl p-6 flex flex-col overflow-hidden ${className}`}>
      <div className="flex items-start justify-between mb-4">
        <div>
          <span className="text-xs font-mono text-muted-darker">{num}</span>
          <h3 className="font-semibold text-lg mt-1">{title}</h3>
          <p className="text-sm text-muted">{subtitle}</p>
        </div>
      </div>
      <div className="flex-1 min-w-0">{children}</div>
    </div>
  );
}

function LogLine({ time, icon, color, label, value }: { time: string; icon: string; color: string; label: string; value: string }) {
  return (
    <div className="flex items-center gap-2 min-w-max">
      <span className="text-muted-darker w-14 shrink-0">{time}</span>
      <span className={`${color} w-3 shrink-0 text-center`}>{icon}</span>
      <span className="text-muted w-24 shrink-0">{label}</span>
      <span className="text-foreground/70">{value}</span>
    </div>
  );
}

/* ─── Config Section ─── */

function ConfigSection() {
  return (
    <section className="py-24 px-6 border-t border-border">
      <div className="max-w-[1200px] mx-auto">
        <div className="max-w-2xl mb-12">
          <p className="text-xs font-mono text-muted-darker uppercase tracking-widest mb-4">Configuration</p>
          <h2 className="text-3xl sm:text-4xl font-bold tracking-[-0.03em] mb-4">
            Declarative policy as YAML
          </h2>
          <p className="text-muted text-lg leading-relaxed">
            Define what&apos;s allowed, denied, and what requires confirmation. Patterns, risk thresholds, and temporal rules. All in a single file.
          </p>
        </div>

        <div className="grid lg:grid-cols-2 gap-3">
          <div className="bg-card border border-border rounded-xl overflow-hidden">
            <div className="flex items-center gap-2 px-5 py-3 border-b border-border">
              <span className="w-3 h-3 rounded-full bg-[#ff5f57]" />
              <span className="w-3 h-3 rounded-full bg-[#febc2e]" />
              <span className="w-3 h-3 rounded-full bg-[#28c840]" />
              <span className="ml-3 text-xs text-muted-darker font-mono">custom-policy.yaml</span>
            </div>
            <div className="p-5 font-mono text-[11px] sm:text-[13px] leading-relaxed overflow-x-auto">
              <Line><K>version</K>: <V>&quot;blackwall/policy-1.0&quot;</V></Line>
              <Line><K>name</K>: <V>&quot;production-deploy&quot;</V></Line>
              <br />
              <Line><K>permissions</K>:</Line>
              <Line indent={1}><K>filesystem</K>:</Line>
              <Line indent={2}><K>read</K>:</Line>
              <Line indent={3}><K>allow</K>: <C>[&quot;$&#123;WORKSPACE&#125;/**&quot;]</C></Line>
              <Line indent={3}><K>deny</K>: <C>[&quot;**/.env&quot;, &quot;**/*.key&quot;]</C></Line>
              <Line indent={2}><K>write</K>:</Line>
              <Line indent={3}><K>allow</K>: <C>[&quot;$&#123;WORKSPACE&#125;/src/**&quot;]</C></Line>
              <Line indent={3}><K>confirm</K>: <C>[&quot;$&#123;WORKSPACE&#125;/config/**&quot;]</C></Line>
              <Line indent={1}><K>shell</K>:</Line>
              <Line indent={2}><K>allow</K>: <C>[git, cargo, npm]</C></Line>
              <Line indent={2}><K>deny</K>: <C>[sudo, su]</C></Line>
              <Line indent={2}><K>confirm</K>: <C>[&quot;rm -rf&quot;, &quot;git push --force&quot;]</C></Line>
              <Line indent={1}><K>network</K>:</Line>
              <Line indent={2}><K>allow</K>: <C>[github.com, crates.io]</C></Line>
              <Line indent={2}><K>deny</K>: <C>[&quot;*&quot;]</C></Line>
            </div>
          </div>

          <div className="flex flex-col gap-3">
            <div className="bg-card border border-border rounded-xl p-6 flex-1">
              <h3 className="font-semibold mb-3">Default Policy</h3>
              <div className="space-y-2.5 text-sm">
                <PolicyRow label="File read" items={[
                  { text: "Workspace files", type: "allow" as const },
                  { text: ".env, secrets, SSH keys", type: "deny" as const },
                ]} />
                <PolicyRow label="File write" items={[
                  { text: "Workspace files", type: "allow" as const },
                  { text: "System dirs, .git/config", type: "deny" as const },
                ]} />
                <PolicyRow label="Shell" items={[
                  { text: "git, npm, cargo...", type: "allow" as const },
                  { text: "sudo, chmod +s, dd", type: "deny" as const },
                ]} />
                <PolicyRow label="Network" items={[
                  { text: "Registries, GitHub", type: "allow" as const },
                  { text: "Everything else", type: "deny" as const },
                ]} />
              </div>
            </div>
            <div className="bg-card border border-border rounded-xl p-6">
              <h3 className="font-semibold mb-2">Blocked by default</h3>
              <p className="text-sm text-muted leading-relaxed mb-4">
                Pipe-to-shell patterns are always blocked, regardless of policy.
              </p>
              <div className="flex flex-wrap gap-2">
                {["curl ... | sh", "wget ... | bash", "sudo rm -rf", "chmod +s", "dd if=", "mkfs"].map((cmd) => (
                  <span key={cmd} className="text-[11px] font-mono px-2.5 py-1 rounded-md bg-red/5 border border-red/10 text-red/80">{cmd}</span>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function Line({ children, indent = 0 }: { children: React.ReactNode; indent?: number }) {
  return <div style={{ paddingLeft: indent * 16 }} className="text-foreground/70">{children}</div>;
}
function K({ children }: { children: React.ReactNode }) {
  return <span className="text-amber">{children}</span>;
}
function V({ children }: { children: React.ReactNode }) {
  return <span className="text-green">{children}</span>;
}
function C({ children }: { children: React.ReactNode }) {
  return <span className="text-foreground/50">{children}</span>;
}

function PolicyRow({ label, items }: { label: string; items: { text: string; type: "allow" | "deny" }[] }) {
  return (
    <div className="flex items-start gap-3">
      <span className="text-muted w-20 shrink-0 pt-0.5">{label}</span>
      <div className="flex flex-wrap gap-1.5">
        {items.map((item) => (
          <span
            key={item.text}
            className={`text-[11px] font-mono px-2 py-0.5 rounded ${
              item.type === "allow"
                ? "bg-green/5 border border-green/10 text-green/80"
                : "bg-red/5 border border-red/10 text-red/80"
            }`}
          >
            {item.text}
          </span>
        ))}
      </div>
    </div>
  );
}

/* ─── Integrations ─── */

function Integrations() {
  return (
    <section id="integrations" className="py-24 px-6 border-t border-border">
      <div className="max-w-[1200px] mx-auto">
        <div className="max-w-2xl mb-12">
          <p className="text-xs font-mono text-muted-darker uppercase tracking-widest mb-4">Integrations</p>
          <h2 className="text-3xl sm:text-4xl font-bold tracking-[-0.03em] mb-4">
            Works with your stack
          </h2>
          <p className="text-muted text-lg leading-relaxed">
            Cursor, Claude Code, or any agent that runs shell commands or MCP tools. Three lines of config.
          </p>
        </div>

        <div className="grid md:grid-cols-3 gap-3">
          <IntegrationCard
            name="Cursor"
            desc="Shell commands intercepted automatically. Wrap MCP servers in your config."
            lines={[
              { text: '// .cursor/mcp.json', dim: true },
              { text: '{' },
              { text: '  "mcpServers": {' },
              { text: '    "fs": {' },
              { text: '      "command": "blackwall",' },
              { text: '      "args": [' },
              { text: '        "proxy-mcp",' },
              { text: '        "--",' },
              { text: '        "npx",' },
              { text: '        "@mcp/server-fs"' },
              { text: '      ]' },
              { text: '    }' },
              { text: '  }' },
              { text: '}' },
            ]}
          />
          <IntegrationCard
            name="Claude Code"
            desc="Wrap the entire process. Every command Claude spawns is intercepted."
            lines={[
              { text: '$ blackwall exec -- claude', highlight: true },
            ]}
          />
          <IntegrationCard
            name="Any Agent"
            desc="Wrap any agent process with full shell and MCP protection."
            lines={[
              { text: '$ blackwall exec -- python agent.py', highlight: true },
              { text: '$ blackwall exec -- node agent.js', highlight: true },
              { text: '' },
              { text: '# Or use the env hook', dim: true },
              { text: '$ source ~/.blackwall/env', highlight: true },
            ]}
          />
        </div>
      </div>
    </section>
  );
}

function IntegrationCard({ name, desc, lines }: {
  name: string;
  desc: string;
  lines: { text: string; dim?: boolean; highlight?: boolean }[];
}) {
  return (
    <div className="bg-card border border-border rounded-xl overflow-hidden flex flex-col">
      <div className="p-6 flex-1">
        <h3 className="font-semibold text-lg mb-1">{name}</h3>
        <p className="text-sm text-muted leading-relaxed">{desc}</p>
      </div>
      <div className="border-t border-border bg-background p-4 font-mono text-xs leading-relaxed whitespace-pre overflow-x-auto">
        {lines.map((line, i) => (
          <div key={i} className={line.dim ? "text-muted-darker" : line.highlight ? "text-foreground/80" : "text-foreground/60"}>
            {line.text || "\u00A0"}
          </div>
        ))}
      </div>
    </div>
  );
}

/* ─── Anti-Patterns ─── */

function AntiPatterns() {
  return (
    <section className="py-24 px-6 border-t border-border">
      <div className="max-w-[1200px] mx-auto">
        <div className="max-w-2xl mb-12">
          <p className="text-xs font-mono text-muted-darker uppercase tracking-widest mb-4">Temporal Detection</p>
          <h2 className="text-3xl sm:text-4xl font-bold tracking-[-0.03em] mb-4">
            Catches what permissions can&apos;t
          </h2>
          <p className="text-muted text-lg leading-relaxed">
            Some attacks look harmless as individual actions. Blackwall detects multi-step sequences over sliding windows, derived from documented frontier model behavior.
          </p>
        </div>

        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-3">
          {[
            { name: "credential_harvesting", desc: "Reading /proc then environment data", action: "HALT", steps: ["/proc/*/environ", "→", ".env"] },
            { name: "sandbox_probing", desc: "Process info → network config → external request", action: "HALT", steps: ["/proc", "→", "ifconfig", "→", "curl"] },
            { name: "self_modification", desc: "Reading then writing MCP/tool configuration", action: "HALT", steps: ["read mcp.json", "→", "write mcp.json"] },
            { name: "public_exfiltration", desc: "Workspace reads then unknown network requests", action: "PAUSE", steps: ["read src/", "→", "POST unknown.com"] },
            { name: "history_tampering", desc: "Git operations then writing .git internals", action: "PAUSE", steps: ["git log", "→", "write .git/"] },
          ].map((p) => (
            <div key={p.name} className="bg-card border border-border rounded-xl p-6">
              <div className="flex items-center justify-between gap-2 mb-3 min-w-0">
                <span className="text-xs font-mono text-muted min-w-0 truncate">{p.name}</span>
                <span className={`text-[10px] font-mono font-medium px-2 py-0.5 rounded shrink-0 ${
                  p.action === "HALT" ? "bg-red/10 text-red border border-red/20" : "bg-amber/10 text-amber border border-amber/20"
                }`}>{p.action}</span>
              </div>
              <p className="text-sm text-muted leading-relaxed mb-4">{p.desc}</p>
              <div className="flex flex-wrap items-center gap-1.5 font-mono text-[10px]">
                {p.steps.map((s, i) => (
                  <span key={i} className={s === "→" ? "text-muted-darker" : "text-foreground/50 bg-surface-tint px-1.5 py-0.5 rounded"}>
                    {s}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ─── Architecture ─── */

function Architecture() {
  return (
    <section className="py-24 px-6 border-t border-border">
      <div className="max-w-[1200px] mx-auto">
        <div className="max-w-2xl mb-12">
          <p className="text-xs font-mono text-muted-darker uppercase tracking-widest mb-4">Architecture</p>
          <h2 className="text-3xl sm:text-4xl font-bold tracking-[-0.03em] mb-4">
            Four layers of evaluation
          </h2>
          <p className="text-muted text-lg leading-relaxed">
            Each action passes through four deterministic stages. Sub-millisecond latency. No inference.
          </p>
        </div>

        <div className="flex flex-col sm:flex-row gap-3">
          {[
            { step: "1", label: "Circuit Breakers", desc: "Session halted? Rate limit exceeded?" },
            { step: "2", label: "Permissions", desc: "Deny → Confirm → Allow → Default deny" },
            { step: "3", label: "Pattern Matching", desc: "Sliding window anti-pattern detection" },
            { step: "4", label: "Risk Scoring", desc: "Cumulative score with pause/halt thresholds" },
          ].map((s, i) => (
            <div key={s.step} className="flex-1 relative bg-card border border-border rounded-xl p-6 flex gap-4 items-start">
              <span className="text-2xl font-bold font-mono text-foreground/10 leading-none shrink-0">{s.step}</span>
              <div>
                <h3 className="font-semibold mb-1.5">{s.label}</h3>
                <p className="text-sm text-muted leading-relaxed">{s.desc}</p>
              </div>
              {i < 3 && (
                <div className="hidden sm:block absolute -right-2 top-1/2 -translate-y-1/2 z-10 text-muted-darker">
                  <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M4 2l4 4-4 4" stroke="currentColor" strokeWidth="1.5"/></svg>
                </div>
              )}
            </div>
          ))}
        </div>

        <div className="mt-12 bg-card border border-border rounded-xl overflow-hidden">
          <div className="flex items-center gap-2 px-5 py-3 border-b border-border">
            <span className="w-3 h-3 rounded-full bg-[#ff5f57]" />
            <span className="w-3 h-3 rounded-full bg-[#febc2e]" />
            <span className="w-3 h-3 rounded-full bg-[#28c840]" />
            <span className="ml-3 text-xs text-muted-darker font-mono">blackwall / gateway</span>
          </div>
          <div className="p-5 font-mono text-[11px] sm:text-[13px] text-foreground/60 leading-relaxed overflow-x-auto">
            <div className="text-foreground/80">■ blackwall v0.1.0</div>
            <div className="pl-2">policy: default | session: 7f3a28</div>
            <div className="pl-2">workspace: /Users/you/project</div>
            <div className="pl-2">log: ~/.blackwall/logs/7f3a28.jsonl</div>
            <br />
            <div className="pl-2">detected: mcp (3 servers), cursor, git</div>
            <br />
            <div className="pl-2"><span className="text-green">●</span> shell hook active, new terminals are protected</div>
            <div className="pl-2">mcp proxy: blackwall proxy-mcp -- &lt;server-command&gt;</div>
            <br />
            <div className="pl-2">gateway active, press ctrl+c to stop</div>
          </div>
        </div>
      </div>
    </section>
  );
}

/* ─── CTA ─── */

function CTA() {
  return (
    <section className="py-14 px-6 relative">
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute bottom-0 left-1/2 -translate-x-1/2 w-[600px] h-[300px] bg-gradient-radial from-foreground/[0.03] to-transparent rounded-full blur-3xl" />
      </div>
      <div className="max-w-[600px] mx-auto text-center relative">
        <h2 className="text-3xl sm:text-4xl font-bold tracking-[-0.03em] mb-4">
          Stop your AI going rogue
        </h2>
        <p className="text-muted text-lg mb-10 leading-relaxed">
          Open source. MIT licensed. Install in under a minute.
        </p>
      </div>
    </section>
  );
}

/* ─── Footer ─── */

function Footer() {
  return (
    <footer className="border-t border-border py-10 px-6">
      <div className="max-w-[1200px] mx-auto flex flex-col sm:flex-row items-center justify-between gap-6">
        <div className="flex items-center gap-4">
          <BrandLogo />
        </div>
        <div className="flex items-center gap-6 text-[13px] text-muted">
          <a href="https://github.com/sambam60/blackwall" target="_blank" rel="noopener noreferrer" className="hover:text-foreground transition-colors">GitHub</a>
          <a href="https://github.com/sambam60/blackwall/blob/main/spec/PROTOCOL.md" target="_blank" rel="noopener noreferrer" className="hover:text-foreground transition-colors">Protocol</a>
          <a href="https://github.com/sambam60/blackwall/blob/main/README.md" target="_blank" rel="noopener noreferrer" className="hover:text-foreground transition-colors">Docs</a>
        </div>
      </div>
    </footer>
  );
}

function BrandLogo() {
  return (
    <span className="inline-flex items-center">
      <img
        src="/blackwall_logo.svg"
        alt="Blackwall"
        className="h-16 w-auto"
        style={{ filter: "var(--logo-filter)" }}
      />
    </span>
  );
}
