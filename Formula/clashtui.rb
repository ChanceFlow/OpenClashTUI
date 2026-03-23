class Clashtui < Formula
  desc "A TUI tool for managing Clash proxies and rules"
  homepage "https://github.com/ChanceFlow/OpenClashTUI"
  version "0.1.7"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.7/clashtui-0.1.7-x86_64-apple-darwin.tar.gz"
      sha256 "7b077fdee03718795dbea7cc0d087742ff6f507141028596826a4827bb514b0d"
    end
    if Hardware::CPU.arm?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.7/clashtui-0.1.7-aarch64-apple-darwin.tar.gz"
      sha256 "e4f754fe2616dd8901638fa67b8c2155fe875b7c127f11c27e4eb1c8b234f281"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.7/clashtui-0.1.7-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "a1e759bfb1662a82835ded4ea70e7b3f935c973acabe9ba6be57eabf002c0392"
    end
    if Hardware::CPU.arm?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.7/clashtui-0.1.7-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "0dd3b418dfe8b6b2bc36865dfb76367aea3e871fe6e3854680148d3841eeff7e"
    end
  end

  def install
    bin.install "clashtui"
  end

  test do
    system "#{bin}/clashtui", "version"
  end
end
