class Candle < Formula
  version "0.2.0"
  desc "Shine a little light on your HTML using the command line"
  homepage "https://github.com/gabebw/candle"

  url "https://github.com/gabebw/candle/releases/download/v#{version}/candle-#{version}.tar.gz"
  sha256 "fdf90f2fa96c30fb8f62768cc755cef69793d1880e810234f9fc0f2857f2c3eb"

  def install
    bin.install "candle"
  end
end
