# typed: false
# frozen_string_literal: true

# This file was generated by GoReleaser. DO NOT EDIT.
class Snazy < Formula
  desc "snazy - a snazzy json log viewer"
  homepage "https://github.com/chmouel/snazy"
  version "0.51.0"

  on_macos do
    url "https://github.com/chmouel/snazy/releases/download/0.51.0/snazy_0.51.0_macOS_all.tar.gz"
    sha256 "2b10cc38a3cc62bc46ba61c7f6446e12284b1780066761b605db0dd10d1ab7cd"

    def install
      bin.install "snazy" => "snazy"
      prefix.install_metafiles

      output = Utils.popen_read("SHELL=bash #{bin}/snazy --shell-completion bash")
      (bash_completion/"snazy").write output

      output = Utils.popen_read("SHELL=zsh #{bin}/snazy --shell-completion zsh")
      (zsh_completion/"_snazy").write output

      output = Utils.popen_read("SHELL=fish #{bin}/snazy --shell-completion fish")
      (fish_completion/"snazy.fish").write output
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/chmouel/snazy/releases/download/0.51.0/snazy_0.51.0_linux_amd64.tar.gz"
      sha256 "e9b72c0778ea8c2259691df98336aaf27d4f83665229be97e16c6382a57616ba"

      def install
        bin.install "snazy" => "snazy"
        prefix.install_metafiles

        output = Utils.popen_read("SHELL=bash #{bin}/snazy --shell-completion bash")
        (bash_completion/"snazy").write output

        output = Utils.popen_read("SHELL=zsh #{bin}/snazy --shell-completion zsh")
        (zsh_completion/"_snazy").write output

        output = Utils.popen_read("SHELL=fish #{bin}/snazy --shell-completion fish")
        (fish_completion/"snazy.fish").write output
      end
    end
  end
end
