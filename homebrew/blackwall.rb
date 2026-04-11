class Blackwall < Formula
  desc "A deterministic execution firewall for AI agents"
  homepage "https://github.com/sambam60/blackwall"
  url "https://github.com/sambam60/blackwall/archive/refs/tags/v0.1.1.tar.gz"
  sha256 "3360a0fe15782bf6fd48ad133115adf39553664a2579d14a5f3fd6589bf45cfa"
  license "MIT"
  head "https://github.com/sambam60/blackwall.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/blackwall-cli")
  end

  def post_install
    ohai "Run 'blackwall init' to set up your shell hook"
    ohai "Then 'blackwall' to start the gateway"
  end

  test do
    assert_match "blackwall", shell_output("#{bin}/blackwall --version")
  end
end
