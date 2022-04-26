# typed: false
# frozen_string_literal: true

# This file was generated by GoReleaser. DO NOT EDIT.
class Snazy < Formula
  desc "snazy - a snazzy json log viewer"
  homepage "https://github.com/chmouel/snazy"
  version "0.3.0"

  on_macos do
    url "https://github.com/chmouel/snazy/releases/download/0.3.0/snazy_0.3.0_macOS_all.tar.gz"
    sha256 "56a3b0e6d4cb4aed23b615b16eac7ecb6184e1eece47a499d600ef33b9859564"

    def install
      bin.install "snazy" => "snazy"
      prefix.install_metafiles
      bash_completion.install "completions/snazy.bash"
      fish_completion.install "completions/snazy.fish"
      zsh_completion.install "completions/_snazy"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/chmouel/snazy/releases/download/0.3.0/snazy_0.3.0_linux_amd64.tar.gz"
      sha256 "7cd72734bc7f33382b07088e418f75a608474f06e4fa72e0e09643f5ef2dded3"

      def install
        bin.install "snazy" => "snazy"
        prefix.install_metafiles
        bash_completion.install "completions/snazy.bash"
        fish_completion.install "completions/snazy.fish"
        zsh_completion.install "completions/_snazy"
      end
    end
  end
end
