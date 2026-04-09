class Blackwall < Formula
  desc "A deterministic execution firewall for AI agents"
  homepage "https://github.com/blackwall-protocol/blackwall"
  url "https://github.com/blackwall-protocol/blackwall/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER"
  license "MIT"
  head "https://github.com/blackwall-protocol/blackwall.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/blackwall-cli")
  end

  test do
    assert_match "blackwall", shell_output("#{bin}/blackwall --version")
  end
end
