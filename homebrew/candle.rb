class Candle < Formula
  version "0.3.0"
  desc "Shine a little light on your HTML using the command line"
  homepage "https://github.com/gabebw/candle"

  url "https://github.com/gabebw/candle/releases/download/v#{version}/candle-#{version}.tar.gz"
  sha256 "cc0e7aa2bb5600992da87d65ea1c39b20d2d7fa1b5cf8bfb2b5f558ba531ae56"

  def install
    bin.install "candle"
  end
end
