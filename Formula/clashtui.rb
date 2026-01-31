class Clashtui < Formula
  desc "A TUI tool for managing Clash proxies and rules"
  homepage "https://github.com/ChanceFlow/OpenClashTUI"
  version "0.1.4"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.4/clashtui-0.1.4-x86_64-apple-darwin.tar.gz"
      sha256 "a8601c6cf0594be7f462f5dae997c25f4e0656ea9905c93e771454a0426b7e0f"
    end
    if Hardware::CPU.arm?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.4/clashtui-0.1.4-aarch64-apple-darwin.tar.gz"
      sha256 "f3f16c9ef9ce473a0f555ba0e29fe5df9eb6162c2b97562ef89ae4b4e2bba120"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.4/clashtui-0.1.4-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "322176016dfe040de0412038d65bdc8f6c9f5147d6a55fa819d40da90be4eb00"
    end
    if Hardware::CPU.arm?
      url "https://github.com/ChanceFlow/OpenClashTUI/releases/download/v0.1.4/clashtui-0.1.4-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "4a0c1652a098086422ef3873b41a0e13cf9bd473175fac511d7a0876dd85dee4"
    end
  end

  def install
    bin.install "clashtui"
  end

  test do
    system "#{bin}/clashtui", "version"
  end
end
