class Clashtui < Formula
  desc "A TUI tool for managing Clash proxies and rules"
  homepage "https://github.com/ChanceFlow/OpenClashTUI"
  version "0.1.6"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.6/clashtui-0.1.6-x86_64-apple-darwin.tar.gz"
      sha256 "73669a2bb69be53a9f8b4dc334c2c80cebe734ae754cf87f96a4be1450e2044a"
    end
    if Hardware::CPU.arm?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.6/clashtui-0.1.6-aarch64-apple-darwin.tar.gz"
      sha256 "028368b0e227463c1e1c1a46b9b40f4043d5c848b9e8fc94e6f18060143bbd9c"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.6/clashtui-0.1.6-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "c5be38f2f3b9007551cdc846f0669db7923db0206ad9bcbc77d815794335a79c"
    end
    if Hardware::CPU.arm?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.6/clashtui-0.1.6-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "33eeae79ff6db29a8ea268e556331bd05317c7868a7832b2c1e564184a95fccd"
    end
  end

  def install
    bin.install "clashtui"
  end

  test do
    system "#{bin}/clashtui", "version"
  end
end
