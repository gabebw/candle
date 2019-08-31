class Candle < Formula
  version "0.1.0"
  desc "Shine a little light on your HTML using the command line"
  homepage "https://github.com/gabebw/candle"

  url "https://github.com/gabebw/candle/releases/download/v#{version}/candle-#{version}.tar.gz"
  sha256 "ed4e1bfd63b935624413abbfc0bcac4a89a5e0788302bd9c26e76b0b17685d85"

  def install
    bin.install "candle"
  end
end
