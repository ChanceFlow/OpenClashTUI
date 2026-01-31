class Clashtui < Formula
  desc "A TUI tool for managing Clash proxies and rules"
  homepage "https://github.com/ChanceFlow/OpenClashTUI"
  version "0.1.5"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.5/clashtui-0.1.5-x86_64-apple-darwin.tar.gz"
      sha256 "7aac0b83fe3c00caa5c3368d7a09ab94cfe2ed311fd9aa9367e8a457a7c3bb2f"
    end
    if Hardware::CPU.arm?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.5/clashtui-0.1.5-aarch64-apple-darwin.tar.gz"
      sha256 "0836c0907c39f719e19207fa62c6d5916eaeb4c42a69ceecbd85699cc6db4ab6"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.5/clashtui-0.1.5-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "b519b1a4c9bad494a39759f201e072e0c5aee451619e2928f7e606f1cd9a753c"
    end
    if Hardware::CPU.arm?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.5/clashtui-0.1.5-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "f9532df6db19b205eb384b16f87a1061fa2aa217b4f30501212ef30c417da0b8"
    end
  end

  def install
    bin.install "clashtui"
  end

  test do
    system "#{bin}/clashtui", "version"
  end
end
