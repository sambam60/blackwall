class Blackwall < Formula
  desc "A deterministic execution firewall for AI agents"
  homepage "https://github.com/sambam60/blackwall"
  url "https://github.com/sambam60/blackwall/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "943b791c1b0ee562f087ca5ebbf7ba459fafcd9f3a694353c85a4a5ff7669dc7"
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
